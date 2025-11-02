#[derive(Debug)]
pub struct HeaderInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression: u8,
    pub filter: u8,
    pub interlace: u8,
}

#[derive(Debug)]
pub struct PaletteInfo {
    pub colors: Vec<(u8, u8, u8)>,
}

#[derive(Debug)]
pub enum AncillaryChunk {
    Gamma { gamma: u32 },
    StandardRGB { rendering_intent: u8 },
    PhysicalIndex { x: u32, y: u32, unit: u8 },
}

#[derive(Debug)]
pub struct PngInfo {
    pub header: HeaderInfo,
    pub palette: Option<PaletteInfo>,
    pub data: Vec<u8>,
    pub ancillary_chunks: Vec<AncillaryChunk>,
}
