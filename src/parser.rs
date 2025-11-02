use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use crate::png::{AncillaryChunk, HeaderInfo, PaletteInfo, PngInfo};

#[derive(Debug)]
pub enum ParserError {
    IOError(io::Error),
    InvalidSignature,
    CorruptedFile,
    MissingHeader,
    MissingPalette,
    MissingData,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::IOError(e) => write!(f, "IO error: {}", e),
            ParserError::CorruptedFile => write!(f, "PNG file is corrupted"),
            ParserError::InvalidSignature => write!(f, "Invalid PNG signature"),
            ParserError::MissingHeader => write!(f, "IHDR chunk is missing"),
            ParserError::MissingPalette => write!(f, "PLTE chunk is missing for indexed colors"),
            ParserError::MissingData => {
                write!(
                    f,
                    "IDAT chunk missing, at least one must present with a single byte"
                )
            }
        }
    }
}

enum ChunkParseResult {
    End,
    Palette(PaletteInfo),
    Data(Vec<u8>),
    Chunk(AncillaryChunk),
    Unknown([u8; 4]),
}

pub struct Parser {
    file: File,
}

impl Parser {
    const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const IHDR: u32 = u32::from_be_bytes(*b"IHDR");
    const IDAT: u32 = u32::from_be_bytes(*b"IDAT");
    const PLTE: u32 = u32::from_be_bytes(*b"PLTE");
    const IEND: u32 = u32::from_be_bytes(*b"IEND");
    const GAMA: u32 = u32::from_be_bytes(*b"gAMA");
    const SRGB: u32 = u32::from_be_bytes(*b"sRGB");
    const PHYS: u32 = u32::from_be_bytes(*b"pHYs");

    pub fn new(filepath: &str) -> Result<Self, ParserError> {
        match File::open(filepath) {
            Ok(file) => Ok(Self { file }),
            Err(e) => Err(ParserError::IOError(e)),
        }
    }

    pub fn parse(&mut self) -> Result<PngInfo, ParserError> {
        self.validate_signature()?;
        let header = self.parse_header()?;

        let mut palette = None;
        let mut data = vec![];
        let mut ancillary_chunks = vec![];

        loop {
            let parse_result = self.parse_chunk()?;
            match parse_result {
                ChunkParseResult::End => {
                    break;
                }
                ChunkParseResult::Palette(p) => {
                    palette = Some(p);
                }
                ChunkParseResult::Data(mut d) => {
                    data.append(&mut d);
                }
                ChunkParseResult::Chunk(chunk) => {
                    ancillary_chunks.push(chunk);
                }
                ChunkParseResult::Unknown(name) => {
                    println!("Unknown chunk: {}", std::str::from_utf8(&name).unwrap());
                }
            };
        }

        if header.color_type == 3 && palette.is_none() {
            return Err(ParserError::MissingPalette);
        }

        if data.len() == 0 {
            return Err(ParserError::MissingData);
        }

        return Ok(PngInfo {
            header,
            palette,
            data,
            ancillary_chunks,
        });
    }

    fn validate_signature(&mut self) -> Result<(), ParserError> {
        let mut signature = [0u8; 8];
        self.read_to(&mut signature)?;
        if signature != Parser::SIGNATURE {
            return Err(ParserError::InvalidSignature);
        }

        return Ok(());
    }

    fn parse_header(&mut self) -> Result<HeaderInfo, ParserError> {
        let length = self.read_u32()?;
        let chunk_type = self.read_u32()?;

        if chunk_type != Parser::IHDR {
            return Err(ParserError::MissingHeader);
        }

        if length != 13 {
            return Err(ParserError::CorruptedFile);
        }

        let mut data = [0u8; 13];
        self.read_to(&mut data)?;

        let header = HeaderInfo {
            width: Parser::to_u32(&data[0..4]),
            height: Parser::to_u32(&data[4..8]),
            bit_depth: data[8],
            color_type: data[9],
            compression: data[10],
            filter: data[11],
            interlace: data[12],
        };

        self.seek(4)?;

        return Ok(header);
    }

    fn parse_chunk(&mut self) -> Result<ChunkParseResult, ParserError> {
        let length = self.read_u32()?;
        let chunk_type = self.read_u32()?;
        let data = self.read_bytes(length as usize)?;

        let chunk = match chunk_type {
            Parser::IEND => {
                if length != 0 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::End
            }
            Parser::IDAT => {
                if length == 0 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::Data(data)
            }
            Parser::PLTE => {
                if length % 3 != 0 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::Palette(PaletteInfo {
                    colors: data
                        .chunks_exact(3)
                        .map(|rgb| (rgb[0], rgb[1], rgb[2]))
                        .collect(),
                })
            }
            Parser::SRGB => {
                if length != 1 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::Chunk(AncillaryChunk::StandardRGB {
                    rendering_intent: data[0],
                })
            }
            Parser::GAMA => {
                if length != 4 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::Chunk(AncillaryChunk::Gamma {
                    gamma: Parser::to_u32(&data[0..4]),
                })
            }
            Parser::PHYS => {
                if length != 9 {
                    return Err(ParserError::CorruptedFile);
                }

                ChunkParseResult::Chunk(AncillaryChunk::PhysicalIndex {
                    x: Parser::to_u32(&data[0..4]),
                    y: Parser::to_u32(&data[4..8]),
                    unit: data[8],
                })
            }
            _ => ChunkParseResult::Unknown(chunk_type.to_be_bytes()),
        };

        self.seek(4)?;

        return Ok(chunk);
    }

    fn to_u32(slice: &[u8]) -> u32 {
        debug_assert_eq!(slice.len(), 4);
        return u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
    }

    fn read_u32(&mut self) -> Result<u32, ParserError> {
        let mut buffer = [0u8; 4];
        match self.file.read_exact(&mut buffer) {
            Ok(()) => Ok(Parser::to_u32(&buffer)),
            Err(e) => Err(ParserError::IOError(e)),
        }
    }

    fn read_to<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], ParserError> {
        match self.file.read_exact(buffer) {
            Ok(()) => Ok(&buffer[..]),
            Err(e) => Err(ParserError::IOError(e)),
        }
    }

    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ParserError> {
        let mut bytes = vec![0u8; size];
        match self.file.read_exact(&mut bytes) {
            Ok(()) => Ok(bytes),
            Err(e) => Err(ParserError::IOError(e)),
        }
    }

    fn seek(&mut self, pos: i64) -> Result<(), ParserError> {
        match self.file.seek(SeekFrom::Current(pos)) {
            Ok(_) => Ok(()),
            Err(e) => Err(ParserError::IOError(e)),
        }
    }
}
