use std::{
    fs::File,
    io::{self, ErrorKind, Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub enum Chunk {
    None,
    Header {
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        compression: u8,
        filter: u8,
        interlace: u8,
    },
    Data {
        data: Vec<u8>,
    },
    Palette {
        colors: Vec<(u8, u8, u8)>,
    },
    End,
    Gamma {
        gamma: u32,
    },
    StandardRGB {
        rendering_intent: u8,
    },
    PhysicalIndex {
        x: u32,
        y: u32,
        unit: u8,
    },
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

    pub fn new(filepath: &String) -> Option<Self> {
        let file = File::open(filepath);
        if let Err(e) = file {
            println!("Unable to read {}: {}", filepath, e);
            return None;
        }

        return Some(Self {
            file: file.unwrap(),
        });
    }

    pub fn parse(&mut self) -> Result<Vec<Chunk>, String> {
        let validation = self.validate_png_signature();
        if let Err(e) = validation {
            return Err(format!("Unable to validate png signature: {}", e));
        }

        if !validation.unwrap() {
            return Err("Provided file is not a PNG file.".into());
        }

        let mut chunks = Vec::<Chunk>::new();

        loop {
            let chunk_result = self.parse_chunk();
            if let Err(ref e) = chunk_result {
                if e.kind() == ErrorKind::UnexpectedEof {
                    break;
                }
            }

            chunks.push(chunk_result.unwrap());
        }

        return Ok(chunks);
    }

    fn validate_png_signature(&mut self) -> io::Result<bool> {
        let mut signature = [0u8; 8];
        self.file.read_exact(&mut signature)?;
        return Ok(signature == Parser::SIGNATURE);
    }

    fn parse_chunk(&mut self) -> io::Result<Chunk> {
        let mut buffer = [0u8; 4];

        self.file.read_exact(&mut buffer)?;
        let length = u32::from_be_bytes(buffer);

        self.file.read_exact(&mut buffer)?;
        let chunk_type = u32::from_be_bytes(buffer);

        let chunk = match chunk_type {
            Parser::IHDR => {
                let mut header = [0u8; 13];
                self.file.read_exact(&mut header)?;

                Chunk::Header {
                    width: u32::from_be_bytes([header[0], header[1], header[2], header[3]]),
                    height: u32::from_be_bytes([header[4], header[5], header[6], header[7]]),
                    bit_depth: header[8],
                    color_type: header[9],
                    compression: header[10],
                    filter: header[11],
                    interlace: header[12],
                }
            }
            Parser::IDAT => {
                let mut data = vec![0u8; length as usize];
                self.file.read_exact(&mut data)?;
                Chunk::Data { data }
            }
            Parser::PLTE => {
                let mut data = vec![0u8; length as usize];
                self.file.read_exact(&mut data)?;

                let colors = data
                    .chunks_exact(3)
                    .map(|rgb| (rgb[0], rgb[1], rgb[2]))
                    .collect();

                Chunk::Palette { colors }
            }
            Parser::IEND => Chunk::End,
            Parser::SRGB => {
                self.file.read_exact(&mut buffer[..1])?;
                let rendering_intent = buffer[0];

                Chunk::StandardRGB { rendering_intent }
            }
            Parser::GAMA => {
                self.file.read_exact(&mut buffer)?;
                let gamma = u32::from_be_bytes(buffer);

                Chunk::Gamma { gamma }
            }
            Parser::PHYS => {
                self.file.read_exact(&mut buffer)?;
                let x = u32::from_be_bytes(buffer);

                self.file.read_exact(&mut buffer)?;
                let y = u32::from_be_bytes(buffer);

                self.file.read_exact(&mut buffer[..1])?;
                let unit = buffer[0];

                Chunk::PhysicalIndex { x, y, unit }
            }
            _ => {
                println!("Unhandled Chunk Type: {}", str::from_utf8(&buffer).unwrap());
                self.file.seek(SeekFrom::Current(length as i64))?;
                Chunk::None
            }
        };

        self.file.seek(SeekFrom::Current(4))?;

        return Ok(chunk);
    }
}
