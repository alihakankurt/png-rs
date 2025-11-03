use std::io::{Read, Seek};

use crate::error::ParserError;
use crate::spec::*;
use crate::utils;

/// Represents a parser that handles PNG data.
pub struct Parser<'a, Source: Read + Seek> {
    source: &'a mut Source,
    header: Option<HeaderInfo>,
    palette: Option<PaletteInfo>,
    compressed_data: Option<CompressedDataInfo>,
    trailer: Option<TrailerInfo>,
    transparency: Option<TransparencyInfo>,
    gamma: Option<GammaInfo>,
    chromaticity: Option<ChromaticityInfo>,
    standard_rgb: Option<StandardRGBInfo>,
    icc_profile: Option<ICCProfileInfo>,
    textual_data: Vec<TextualDataInfo>,
    compressed_textual_data: Vec<CompressedTextualDataInfo>,
    international_textual_data: Vec<InternationalTextualDataInfo>,
    background: Option<BackgroundInfo>,
    physical_pixel_dimension: Option<PhysicalPixelDimensionInfo>,
    significant_bits: Option<SignificantBitsInfo>,
    suggested_palettes: Vec<SuggestedPaletteInfo>,
    palette_histogram: Option<PaletteHistogramInfo>,
    last_modification: Option<LastModificationInfo>,
    unknown_chunks: Vec<UnknownChunkInfo>,
}

impl<'a, Source: Read + Seek> Parser<'a, Source> {
    const BEFORE_PLTE_CHUNK: u8 = 1;
    const AFTER_PLTE_CHUNK: u8 = 2;
    const BEFORE_IDAT_CHUNK: u8 = 4;

    /// Tries to parse PNG data from provided source.
    pub fn parse(source: &'a mut Source) -> Result<PngInfo, ParserError> {
        let mut parser = Self {
            source,
            header: None,
            palette: None,
            compressed_data: None,
            trailer: None,
            transparency: None,
            gamma: None,
            chromaticity: None,
            standard_rgb: None,
            icc_profile: None,
            textual_data: Vec::new(),
            compressed_textual_data: Vec::new(),
            international_textual_data: Vec::new(),
            background: None,
            physical_pixel_dimension: None,
            significant_bits: None,
            suggested_palettes: Vec::new(),
            palette_histogram: None,
            last_modification: None,
            unknown_chunks: Vec::new(),
        };

        parser.validate_signature()?;
        parser.parse_header()?;
        parser.parse_chunks()?;

        return parser.collect();
    }

    fn validate_signature(&mut self) -> Result<(), ParserError> {
        let mut signature = [0u8; 8];
        utils::read_to(self.source, &mut signature)?;
        if signature != SIGNATURE {
            return Err(ParserError::InvalidSignature);
        }

        return Ok(());
    }

    fn parse_header(&mut self) -> Result<(), ParserError> {
        let length = utils::read_u32(self.source)?;
        let type_and_data = utils::read_bytes(self.source, 4 + length as usize)?;
        // TODO(@alihakankurt): Use this variable to check data integrity.
        let _crc = utils::read_u32(self.source)?;

        let chunk_type = utils::to_u32(&type_and_data[..4]);
        let data = &type_and_data[4..];

        if chunk_type != chunk_ids::IHDR {
            return Err(ParserError::MissingRequiredChunk(chunk_ids::IHDR));
        }

        if length != 13 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::IHDR));
        }

        let header_info = HeaderInfo {
            width: utils::to_u32(&data[0..4]),
            height: utils::to_u32(&data[4..8]),
            bit_depth: data[8],
            color_type: match data[9] {
                0 => ColorType::Grayscale,
                2 => ColorType::TrueColor,
                3 => ColorType::IndexedColor,
                4 => ColorType::GrayscaleAlpha,
                6 => ColorType::TrueColorAlpha,
                _ => return Err(ParserError::InvalidFieldValue),
            },
            compression_method: match data[10] {
                0 => CompressionMethod::Deflate,
                _ => return Err(ParserError::InvalidFieldValue),
            },
            filter_method: match data[11] {
                0 => FilterMethod::Adaptive,
                _ => return Err(ParserError::InvalidFieldValue),
            },
            interlace_method: match data[12] {
                0 => InterlaceMethod::None,
                1 => InterlaceMethod::Adam7,
                _ => return Err(ParserError::InvalidFieldValue),
            },
        };

        if header_info.width == 0 || header_info.height == 0 {
            return Err(ParserError::InvalidFieldValue);
        }

        let is_valid_bit_depth = match header_info.color_type {
            ColorType::Grayscale => matches!(header_info.bit_depth, 1 | 2 | 4 | 8 | 16),
            ColorType::IndexedColor => matches!(header_info.bit_depth, 1 | 2 | 4 | 8),
            ColorType::TrueColor | ColorType::GrayscaleAlpha | ColorType::TrueColorAlpha => {
                matches!(header_info.bit_depth, 8 | 16)
            }
        };

        if !is_valid_bit_depth {
            return Err(ParserError::InvalidFieldValue);
        }

        self.header = Some(header_info);

        return Ok(());
    }

    fn parse_chunks(&mut self) -> Result<(), ParserError> {
        while self.trailer.is_none() {
            let length = utils::read_u32(self.source)?;
            let type_and_data = utils::read_bytes(self.source, 4 + length as usize)?;
            // TODO(@alihakankurt): Use this variable to check data integrity.
            let _crc = utils::read_u32(self.source)?;

            let chunk_type = utils::to_u32(&type_and_data[..4]);
            let data = &type_and_data[4..];

            match chunk_type {
                chunk_ids::PLTE => self.parse_plte(length, data)?,
                chunk_ids::IDAT => self.parse_idat(length, data)?,
                chunk_ids::IEND => self.parse_iend(length, data)?,
                chunk_ids::tRNS => self.parse_trns(length, data)?,
                chunk_ids::gAMA => self.parse_gama(length, data)?,
                chunk_ids::cHRM => self.parse_chrm(length, data)?,
                chunk_ids::sRGB => self.parse_srgb(length, data)?,
                chunk_ids::iCCP => self.parse_iccp(length, data)?,
                chunk_ids::tEXt => self.parse_text(length, data)?,
                chunk_ids::zTXt => self.parse_ztxt(length, data)?,
                chunk_ids::iTXt => self.parse_itxt(length, data)?,
                chunk_ids::bKGD => self.parse_bkgd(length, data)?,
                chunk_ids::pHYs => self.parse_phys(length, data)?,
                chunk_ids::sBIT => self.parse_sbit(length, data)?,
                chunk_ids::sPLT => self.parse_splt(length, data)?,
                chunk_ids::hIST => self.parse_hist(length, data)?,
                chunk_ids::tIME => self.parse_time(length, data)?,
                _ => {
                    self.unknown_chunks.push(UnknownChunkInfo {
                        chunk_type: chunk_type.to_be_bytes(),
                        data: Vec::from(data),
                    });
                }
            };
        }

        return Ok(());
    }

    fn parse_plte(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.palette.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::PLTE));
        }

        self.check_chunk_order(chunk_ids::PLTE, Self::BEFORE_IDAT_CHUNK)?;

        if length % 3 != 0 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::PLTE));
        }

        self.palette = Some(PaletteInfo {
            entries: utils::to_chunked::<3, _, _>(&data, |c| (c[0], c[1], c[2])),
        });

        return Ok(());
    }

    fn parse_idat(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.compressed_data.is_none() {
            return Err(ParserError::NonConsecutiveData);
        }

        if length == 0 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::IDAT));
        }

        let mut data = Vec::from(data);
        let mut chunk_count = 1;

        loop {
            let length = utils::read_u32(self.source)?;
            let chunk_type = utils::read_u32(self.source)?;

            if chunk_type != chunk_ids::IDAT {
                utils::seek(self.source, -8)?;
                break;
            }

            if length == 0 {
                return Err(ParserError::InvalidChunkLength(chunk_ids::IDAT));
            }

            utils::seek(self.source, -4)?;
            let type_and_data = utils::read_bytes(self.source, 4 + length as usize)?;
            // TODO(@alihakankurt): Use this variable to check data integrity.
            let _crc = utils::read_u32(self.source)?;

            data.extend_from_slice(&type_and_data[4..]);
            chunk_count += 1;
        }

        self.compressed_data = Some(CompressedDataInfo { chunk_count, data });

        return Ok(());
    }

    fn parse_iend(&mut self, length: u32, _data: &[u8]) -> Result<(), ParserError> {
        if !self.trailer.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::IEND));
        }

        if length != 0 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::IEND));
        }

        self.trailer = Some(TrailerInfo { found: true });

        return Ok(());
    }

    fn parse_trns(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.transparency.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::tRNS));
        }

        self.check_chunk_order(
            chunk_ids::tRNS,
            Self::AFTER_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        let header = self.header.as_ref().unwrap();

        let transparency = match header.color_type {
            ColorType::Grayscale => {
                if length != 2 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::tRNS));
                }

                TransparencyVariant::Grayscale(utils::to_u16(&data[0..2]))
            }
            ColorType::TrueColor => {
                if length != 6 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::tRNS));
                }

                TransparencyVariant::TrueColor(
                    utils::to_u16(&data[0..2]),
                    utils::to_u16(&data[2..4]),
                    utils::to_u16(&data[4..6]),
                )
            }
            ColorType::IndexedColor => {
                let palette = self.palette.as_ref().unwrap();
                if length != palette.entries.len() as u32 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::tRNS));
                }

                TransparencyVariant::IndexedColor(Vec::from(data))
            }
            _ => return Err(ParserError::InvalidFieldValue),
        };

        self.transparency = Some(TransparencyInfo { transparency });

        return Ok(());
    }

    fn parse_gama(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.gamma.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::gAMA));
        }

        self.check_chunk_order(
            chunk_ids::gAMA,
            Self::BEFORE_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        if length != 4 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::gAMA));
        }

        let gamma = utils::to_f32(data);

        self.gamma = Some(GammaInfo { gamma });

        return Ok(());
    }

    fn parse_chrm(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.chromaticity.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::cHRM));
        }

        self.check_chunk_order(
            chunk_ids::cHRM,
            Self::BEFORE_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        if length != 32 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::cHRM));
        }

        let white_point = (utils::to_f32(&data[0..4]), utils::to_f32(&data[4..8]));
        let red = (utils::to_f32(&data[8..12]), utils::to_f32(&data[12..16]));
        let green = (utils::to_f32(&data[16..20]), utils::to_f32(&data[20..24]));
        let blue = (utils::to_f32(&data[24..28]), utils::to_f32(&data[28..32]));

        self.chromaticity = Some(ChromaticityInfo {
            white_point,
            red,
            green,
            blue,
        });

        return Ok(());
    }

    fn parse_srgb(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.standard_rgb.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::sRGB));
        }

        self.check_chunk_order(
            chunk_ids::sRGB,
            Self::BEFORE_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        if length != 1 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::sRGB));
        }

        let rendering_intent = match data[0] {
            0 => RenderingIntent::Perceptual,
            1 => RenderingIntent::RelativeColorimetric,
            2 => RenderingIntent::Saturation,
            3 => RenderingIntent::AbsoluteColorimetric,
            _ => return Err(ParserError::InvalidFieldValue),
        };

        self.standard_rgb = Some(StandardRGBInfo { rendering_intent });

        return Ok(());
    }

    fn parse_iccp(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.icc_profile.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::iCCP));
        }

        self.check_chunk_order(
            chunk_ids::iCCP,
            Self::BEFORE_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        if length < 4 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::iCCP));
        }

        let name = utils::get_string(data)?;
        utils::validate_string(&name)?;
        let data = &data[name.len()..];

        let compression_method = match data[0] {
            0 => CompressionMethod::Deflate,
            _ => return Err(ParserError::InvalidFieldValue),
        };
        let data = &data[1..];

        let compressed_profile_data = Vec::from(data);

        self.icc_profile = Some(ICCProfileInfo {
            name,
            compression_method,
            compressed_profile_data,
        });

        return Ok(());
    }

    fn parse_text(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if length < 2 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::tEXt));
        }

        let keyword = utils::get_string(data)?;
        utils::validate_string(&keyword)?;
        let data = &data[keyword.len() + 1..];

        let text = utils::to_string(data);

        self.textual_data.push(TextualDataInfo { keyword, text });

        return Ok(());
    }

    fn parse_ztxt(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if length < 3 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::zTXt));
        }

        let keyword = utils::get_string(data)?;
        utils::validate_string(&keyword)?;
        let data = &data[keyword.len() + 1..];

        let compression_method = match data[0] {
            0 => CompressionMethod::Deflate,
            _ => return Err(ParserError::InvalidFieldValue),
        };
        let data = &data[1..];

        let text = Vec::from(data);

        self.compressed_textual_data
            .push(CompressedTextualDataInfo {
                keyword,
                compression_method,
                text,
            });

        return Ok(());
    }

    fn parse_itxt(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if length < 6 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::iTXt));
        }

        let keyword = utils::get_string(data)?;
        utils::validate_string(&keyword)?;
        let data = &data[keyword.len() + 1..];

        let is_compressed = data[0] == 1;
        let compression_method = match data[1] {
            0 => CompressionMethod::Deflate,
            _ => return Err(ParserError::InvalidFieldValue),
        };
        let data = &data[2..];

        let language_tag = utils::get_string(data)?;
        let data = &data[language_tag.len() + 1..];

        let translated_keyword = utils::get_string(data)?;
        let data = &data[translated_keyword.len() + 1..];

        let text = Vec::from(data);

        self.international_textual_data
            .push(InternationalTextualDataInfo {
                keyword,
                is_compressed,
                compression_method,
                language_tag,
                translated_keyword,
                text,
            });

        return Ok(());
    }

    fn parse_bkgd(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.background.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::bKGD));
        }

        self.check_chunk_order(
            chunk_ids::bKGD,
            Self::AFTER_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        let header = self.header.as_ref().unwrap();

        let background = match header.color_type {
            ColorType::Grayscale => {
                if length != 2 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::bKGD));
                }

                BackgroundVariant::Grayscale(utils::to_u16(&data[0..2]))
            }
            ColorType::TrueColor => {
                if length != 6 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::bKGD));
                }

                BackgroundVariant::TrueColor(
                    utils::to_u16(&data[0..2]),
                    utils::to_u16(&data[2..4]),
                    utils::to_u16(&data[4..6]),
                )
            }
            ColorType::IndexedColor => {
                if length != 1 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::bKGD));
                }

                BackgroundVariant::IndexedColor(data[0])
            }
            _ => return Err(ParserError::InvalidFieldValue),
        };

        self.background = Some(BackgroundInfo { background });

        return Ok(());
    }

    fn parse_phys(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.physical_pixel_dimension.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::pHYs));
        }

        self.check_chunk_order(chunk_ids::pHYs, Self::BEFORE_IDAT_CHUNK)?;

        if length != 9 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::pHYs));
        }

        let pixels_per_unit = (utils::to_u32(&data[0..4]), utils::to_u32(&data[4..8]));
        let unit_specifier = match data[8] {
            0 => PhysicalUnitSpecifier::Unknown,
            1 => PhysicalUnitSpecifier::Meter,
            _ => return Err(ParserError::InvalidFieldValue),
        };

        self.physical_pixel_dimension = Some(PhysicalPixelDimensionInfo {
            pixels_per_unit,
            unit_specifier,
        });

        return Ok(());
    }

    fn parse_sbit(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.significant_bits.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::sBIT));
        }

        self.check_chunk_order(
            chunk_ids::sBIT,
            Self::BEFORE_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        let header = self.header.as_ref().unwrap();

        let variant = match header.color_type {
            ColorType::Grayscale => {
                if length != 1 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sBIT));
                }

                SignificantBitsVariant::Grayscale(data[0])
            }
            ColorType::TrueColor => {
                if length != 3 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sBIT));
                }

                SignificantBitsVariant::TrueColor(data[0], data[1], data[2])
            }
            ColorType::IndexedColor => {
                if length != 3 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sBIT));
                }

                SignificantBitsVariant::IndexedColor(data[0], data[1], data[2])
            }
            ColorType::GrayscaleAlpha => {
                if length != 2 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sBIT));
                }

                SignificantBitsVariant::GrayscaleAlpha(data[0], data[1])
            }
            ColorType::TrueColorAlpha => {
                if length != 4 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sBIT));
                }

                SignificantBitsVariant::TrueColorAlpha(data[0], data[1], data[2], data[3])
            }
        };

        self.significant_bits = Some(SignificantBitsInfo {
            significant_bits: variant,
        });

        return Ok(());
    }

    fn parse_splt(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        self.check_chunk_order(chunk_ids::sPLT, Self::BEFORE_IDAT_CHUNK)?;

        if length < 4 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::sPLT));
        }

        let name = utils::get_string(data)?;
        utils::validate_string(&name)?;
        let data = &data[name.len() + 1..];

        let sample_depth = data[0];
        let data = &data[1..];

        let entries = match sample_depth {
            1 => {
                if data.len() % 6 != 0 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sPLT));
                }

                utils::to_chunked::<6, _, _>(data, |c| SuggestedPaletteEntry {
                    red: c[0] as u16,
                    green: c[1] as u16,
                    blue: c[2] as u16,
                    alpha: c[3] as u16,
                    frequency: utils::to_u16(&c[4..6]),
                })
            }
            2 => {
                if data.len() % 10 != 0 {
                    return Err(ParserError::InvalidChunkLength(chunk_ids::sPLT));
                }

                utils::to_chunked::<10, _, _>(data, |c| SuggestedPaletteEntry {
                    red: utils::to_u16(&c[0..2]),
                    green: utils::to_u16(&c[2..4]),
                    blue: utils::to_u16(&c[4..6]),
                    alpha: utils::to_u16(&c[6..8]),
                    frequency: utils::to_u16(&c[8..10]),
                })
            }
            _ => return Err(ParserError::InvalidFieldValue),
        };

        self.suggested_palettes.push(SuggestedPaletteInfo {
            name,
            sample_depth,
            entries,
        });

        return Ok(());
    }

    fn parse_hist(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.palette_histogram.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::hIST));
        }

        self.check_chunk_order(
            chunk_ids::hIST,
            Self::AFTER_PLTE_CHUNK | Self::BEFORE_IDAT_CHUNK,
        )?;

        let palette = self.palette.as_ref().unwrap();

        if length != (palette.entries.len() * 2) as u32 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::hIST));
        }

        let entries = utils::to_chunked::<2, _, _>(data, |c| utils::to_u16(c));

        self.palette_histogram = Some(PaletteHistogramInfo { entries });

        return Ok(());
    }

    fn parse_time(&mut self, length: u32, data: &[u8]) -> Result<(), ParserError> {
        if !self.last_modification.is_none() {
            return Err(ParserError::DuplicateChunk(chunk_ids::tIME));
        }

        if length != 7 {
            return Err(ParserError::InvalidChunkLength(chunk_ids::tIME));
        }

        let year = utils::to_u16(&data[0..2]);
        let month = data[2];
        let day = data[3];
        let hour = data[4];
        let minute = data[5];
        let second = data[6];

        self.last_modification = Some(LastModificationInfo {
            year,
            month,
            day,
            hour,
            minute,
            second,
        });

        return Ok(());
    }

    fn check_chunk_order(&self, chunk_id: ChunkId, constraint: u8) -> Result<(), ParserError> {
        if (constraint & Self::BEFORE_PLTE_CHUNK) != 0 && !self.palette.is_none() {
            return Err(ParserError::InvalidChunkOrder(chunk_id));
        }

        if (constraint & Self::AFTER_PLTE_CHUNK) != 0 && self.palette.is_none() {
            return Err(ParserError::InvalidChunkOrder(chunk_id));
        }

        if (constraint & Self::BEFORE_IDAT_CHUNK) != 0 && !self.compressed_data.is_none() {
            return Err(ParserError::InvalidChunkOrder(chunk_id));
        }

        return Ok(());
    }

    fn collect(self) -> Result<PngInfo, ParserError> {
        let header = self.header.unwrap();
        let palette = match self.palette {
            Some(palette) => Some(palette),
            None => {
                if let ColorType::IndexedColor = header.color_type {
                    return Err(ParserError::MissingRequiredChunk(chunk_ids::PLTE));
                }
                None
            }
        };
        let compressed_data = match self.compressed_data {
            Some(data) => data,
            None => return Err(ParserError::MissingRequiredChunk(chunk_ids::IDAT)),
        };
        let trailer = match self.trailer {
            Some(trailer) => {
                if !trailer.found {
                    return Err(ParserError::MissingRequiredChunk(chunk_ids::IEND));
                }
                trailer
            }
            None => return Err(ParserError::MissingRequiredChunk(chunk_ids::IEND)),
        };

        return Ok(PngInfo {
            header,
            compressed_data,
            palette,
            trailer,
            transparency: self.transparency,
            gamma: self.gamma,
            chromaticity: self.chromaticity,
            standard_rgb: self.standard_rgb,
            icc_profile: self.icc_profile,
            textual_data: self.textual_data,
            compressed_textual_data: self.compressed_textual_data,
            international_textual_data: self.international_textual_data,
            background: self.background,
            physical_pixel_dimension: self.physical_pixel_dimension,
            significant_bits: self.significant_bits,
            suggested_palettes: self.suggested_palettes,
            palette_histogram: self.palette_histogram,
            last_modification: self.last_modification,
            unknown_chunks: self.unknown_chunks,
        });
    }
}
