/// The byte values of PNG signature.
pub const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// The type of each chunk id.
pub type ChunkId = u32;

#[allow(non_upper_case_globals)]
pub mod chunk_ids {
    use crate::spec::ChunkId;

    /// Image Header
    pub const IHDR: ChunkId = u32::from_be_bytes(*b"IHDR");
    /// Palette
    pub const PLTE: ChunkId = u32::from_be_bytes(*b"PLTE");
    /// Image Data
    pub const IDAT: ChunkId = u32::from_be_bytes(*b"IDAT");
    /// Image Trailer
    pub const IEND: ChunkId = u32::from_be_bytes(*b"IEND");
    /// Transparency
    pub const tRNS: ChunkId = u32::from_be_bytes(*b"tRNS");
    /// Image Gamma
    pub const gAMA: ChunkId = u32::from_be_bytes(*b"gAMA");
    /// Primary Chromaticities
    pub const cHRM: ChunkId = u32::from_be_bytes(*b"cHRM");
    /// Standard RGB Color Space
    pub const sRGB: ChunkId = u32::from_be_bytes(*b"sRGB");
    /// Embedded ICC Profile
    pub const iCCP: ChunkId = u32::from_be_bytes(*b"iCCP");
    /// Textual Data
    pub const tEXt: ChunkId = u32::from_be_bytes(*b"tEXt");
    /// Compressed Textual Data
    pub const zTXt: ChunkId = u32::from_be_bytes(*b"zTXt");
    /// International Textual Data
    pub const iTXt: ChunkId = u32::from_be_bytes(*b"iTXt");
    /// Background Color
    pub const bKGD: ChunkId = u32::from_be_bytes(*b"bKGD");
    /// Physical Pixel Dimensions
    pub const pHYs: ChunkId = u32::from_be_bytes(*b"pHYs");
    /// Significant Bits
    pub const sBIT: ChunkId = u32::from_be_bytes(*b"sBIT");
    /// Suggested Palette
    pub const sPLT: ChunkId = u32::from_be_bytes(*b"sPLT");
    /// Palette Histogram
    pub const hIST: ChunkId = u32::from_be_bytes(*b"hIST");
    /// Image Last Modification Time
    pub const tIME: ChunkId = u32::from_be_bytes(*b"tIME");
}

/// Describes the pixel interpretation of an image data.
#[derive(Debug)]
pub enum ColorType {
    /// Each pixel is a grayscale sample.
    Grayscale,
    /// Each pixel is an RGB triplet.
    TrueColor,
    /// Each pixel is palette index; a PLTE chunk must appear.
    IndexedColor,
    /// Each pixel is a grayscale sample, followed by an alpha sample.
    GrayscaleAlpha,
    /// Each pixel is an RGB triplet, followed by an alpha sample.
    TrueColorAlpha,
}

/// Describes the compression method used to compress data.
#[derive(Debug)]
pub enum CompressionMethod {
    /// Deflate/Inflate compression with a sliding window of at most 32768 (2^15) bytes.
    Deflate,
}

/// Describes the preprocessing method applied to the image data before compression.
#[derive(Debug)]
pub enum FilterMethod {
    /// Adaptive filtering with five basic filter types.
    Adaptive,
}

/// Describes the transmission order of the image data.
#[derive(Debug)]
pub enum InterlaceMethod {
    /// No interlace,
    None,
    /// Adam7 interlace.
    Adam7,
}

/// Represents the info of `IHDR` chunk.
#[derive(Debug)]
pub struct HeaderInfo {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The number of bits per sample.
    pub bit_depth: u8,
    /// The color type.
    pub color_type: ColorType,
    /// The compression method.
    pub compression_method: CompressionMethod,
    /// The filter method.
    pub filter_method: FilterMethod,
    /// The interlace method.
    pub interlace_method: InterlaceMethod,
}

/// Represents the info of `PLTE` chunk.
#[derive(Debug)]
pub struct PaletteInfo {
    /// The colors in the form of an RGB triplet.
    pub entries: Vec<(u8, u8, u8)>,
}

/// Represents the info of `IDAT` chunk.
#[derive(Debug)]
pub struct CompressedDataInfo {
    /// The number of chunks.
    pub chunk_count: u32,
    /// The compressed pixel data.
    pub data: Vec<u8>,
}

/// Represents the info of `IEND` chunk.
#[derive(Debug)]
pub struct TrailerInfo {
    /// Whether the chunk is found or not.
    pub found: bool,
}

/// Describes the transparency.
#[derive(Debug)]
pub enum TransparencyVariant {
    /// For grayscale images, a single gray level value.
    Grayscale(u16),
    /// For true-color images, an RGB color value.
    TrueColor(u16, u16, u16),
    /// For indexed-color images, series of alpha values corresponding to the palette entries.
    IndexedColor(Vec<u8>),
}

/// Represents the info of `tRNS` chunk.
#[derive(Debug)]
pub struct TransparencyInfo {
    /// The transparency variant.
    pub transparency: TransparencyVariant,
}

/// Represents the info of `gAMA` chunk.
#[derive(Debug)]
pub struct GammaInfo {
    /// The gamma value.
    pub gamma: f32,
}

/// Represents the info of `cHRM` chunk.
#[derive(Debug)]
pub struct ChromaticityInfo {
    /// The white point chromaticity of X and Y axes.
    pub white_point: (f32, f32),
    /// The red chromaticity of X and Y axes.
    pub red: (f32, f32),
    /// The green chromaticity of X and Y axes.
    pub green: (f32, f32),
    /// The blue chromaticity of X and Y axes.
    pub blue: (f32, f32),
}

/// Describes the rendering intent image.
#[derive(Debug)]
pub enum RenderingIntent {
    /// Images preferring good adaptation to the output device gamut at the expense of colorimetric accuracy, like photographs.
    Perceptual,
    /// Images requiring color appearance matching (relative to the output device white point), like logos.
    RelativeColorimetric,
    /// Images preferring preservation of saturation at the expense of hue and lightness, like charts and graphs.
    Saturation,
    /// Images requiring preservation of absolute colorimetry, like proofs (previews of images destined for a different output device).
    AbsoluteColorimetric,
}

/// Represents the info of `sRGB` chunk.
#[derive(Debug)]
pub struct StandardRGBInfo {
    /// The rendering intent.
    pub rendering_intent: RenderingIntent,
}

/// Represents the info of `iCCP` chunk.
#[derive(Debug)]
pub struct ICCProfileInfo {
    /// The profile name.
    pub name: String,
    /// The compression method used to compress profile data.
    pub compression_method: CompressionMethod,
    /// The compressed profile data.
    pub compressed_profile_data: Vec<u8>,
}

/// Represents the info of `tEXt` chunk.
#[derive(Debug)]
pub struct TextualDataInfo {
    /// The keyword.
    pub keyword: String,
    /// The text.
    pub text: String,
}

/// Represents the info of `zTXt` chunk.
#[derive(Debug)]
pub struct CompressedTextualDataInfo {
    /// The keyword.
    pub keyword: String,
    /// The compression method used to compress text.
    pub compression_method: CompressionMethod,
    /// The compressed text data.
    pub text: Vec<u8>,
}

/// Represents the info of `iTXt` chunk.
#[derive(Debug)]
pub struct InternationalTextualDataInfo {
    /// The keyword.
    pub keyword: String,
    /// Whether the text is compressed.
    pub is_compressed: bool,
    /// The compression method used to compress text.
    pub compression_method: CompressionMethod,
    /// The language.
    pub language_tag: String,
    /// The translated keyword.
    pub translated_keyword: String,
    /// The international (maybe compressed) text data.
    pub text: Vec<u8>,
}

/// Describes the default background color of image.
#[derive(Debug)]
pub enum BackgroundVariant {
    /// For grayscale images, a single value as gray level.
    Grayscale(u16),
    /// For true-color images, an RGB color.
    TrueColor(u16, u16, u16),
    /// For indexed-color images, a palette index.
    IndexedColor(u8),
}

/// Represents the info of `bKGD` chunk.
#[derive(Debug)]
pub struct BackgroundInfo {
    /// The background variant.
    pub background: BackgroundVariant,
}

/// Describes physical pixel unit specifier.
#[derive(Debug)]
pub enum PhysicalUnitSpecifier {
    /// Unit is unknown, used to define pixel aspect ratio only.
    Unknown,
    /// Unit is in meters.
    Meter,
}

/// Represents the info of `pHYs` chunk.
#[derive(Debug)]
pub struct PhysicalPixelDimensionInfo {
    /// The pixels per unit in X and Y axes.
    pub pixels_per_unit: (u32, u32),
    /// The physical unit specifier.
    pub unit_specifier: PhysicalUnitSpecifier,
}

/// Describes the significant bits.
#[derive(Debug)]
pub enum SignificantBitsVariant {
    /// For grayscale images, a single byte, indicating the number of bits that were significant in the source data.
    Grayscale(u8),
    /// For true-color images, three bytes, indicating the number of bits that were significant in the source data for the red, green, and blue channels, respectively.
    TrueColor(u8, u8, u8),
    /// For indexed-colors, three bytes, indicating the number of bits that were significant in the source data for the red, green, and blue components of the palette entries, respectively.
    IndexedColor(u8, u8, u8),
    /// For grayscale images with alpha channel, two bytes, indicating the number of bits that were significant in the source grayscale data and the source alpha data, respectively.
    GrayscaleAlpha(u8, u8),
    /// For true-color images with alpha channel, four bytes, indicating the number of bits that were significant in the source data for the red, green, blue, and alpha channels, respectively.
    TrueColorAlpha(u8, u8, u8, u8),
}

/// Represents the info of `sBIT` chunk.
#[derive(Debug)]
pub struct SignificantBitsInfo {
    /// The significant bits.
    pub significant_bits: SignificantBitsVariant,
}

/// Represents an entry for suggested palette info.
#[derive(Debug)]
pub struct SuggestedPaletteEntry {
    /// The red sample.
    pub red: u16,
    /// The green sample.
    pub green: u16,
    /// The blue sample.
    pub blue: u16,
    /// The alpha sample.
    pub alpha: u16,
    /// The frequency.
    pub frequency: u16,
}

/// Represents the info of `sPLT` chunk.
#[derive(Debug)]
pub struct SuggestedPaletteInfo {
    /// The palette name.
    pub name: String,
    /// The sample depth
    pub sample_depth: u8,
    /// The sample entries.
    pub entries: Vec<SuggestedPaletteEntry>,
}

/// Represents the info of `hIST` chunk.
#[derive(Debug)]
pub struct PaletteHistogramInfo {
    /// The histogram entries corresponding to the palette entries.
    pub entries: Vec<u16>,
}

/// Represents the info of `tIME` chunk.
#[derive(Debug)]
pub struct LastModificationInfo {
    /// The last modified full year.
    pub year: u16,
    /// The last modified month. (1-12)
    pub month: u8,
    /// The last modified day. (1-31)
    pub day: u8,
    /// The last modified hour. (0-23)
    pub hour: u8,
    /// The last modified minute. (0-59)
    pub minute: u8,
    /// The last modified second. (0-60; 60 for leap seconds)
    pub second: u8,
}

/// Represents the info of an unknown chunk.
#[derive(Debug)]
pub struct UnknownChunkInfo {
    /// The chunk type.
    pub chunk_type: [u8; 4],
    /// The raw data.
    pub data: Vec<u8>,
}

/// Represents the info of a PNG image.
#[derive(Debug)]
pub struct PngInfo {
    /// The header.
    pub header: HeaderInfo,
    /// The palette.
    pub palette: Option<PaletteInfo>,
    /// The compressed data.
    pub compressed_data: CompressedDataInfo,
    /// The trailer.
    pub trailer: TrailerInfo,
    /// The transparency values.
    pub transparency: Option<TransparencyInfo>,
    /// The gamma value.
    pub gamma: Option<GammaInfo>,
    /// The primary chromaticities.
    pub chromaticity: Option<ChromaticityInfo>,
    /// The standard rgb.
    pub standard_rgb: Option<StandardRGBInfo>,
    /// The ICC profile.
    pub icc_profile: Option<ICCProfileInfo>,
    /// The vector of textual data.
    pub textual_data: Vec<TextualDataInfo>,
    /// The vector of compressed textual data.
    pub compressed_textual_data: Vec<CompressedTextualDataInfo>,
    /// The vector of international textual data.
    pub international_textual_data: Vec<InternationalTextualDataInfo>,
    /// The background color.
    pub background: Option<BackgroundInfo>,
    /// The physical pixel dimension.
    pub physical_pixel_dimension: Option<PhysicalPixelDimensionInfo>,
    /// The significant bits.
    pub significant_bits: Option<SignificantBitsInfo>,
    /// The vector of suggested palettes.
    pub suggested_palettes: Vec<SuggestedPaletteInfo>,
    /// The histogram of the palette if exists.
    pub palette_histogram: Option<PaletteHistogramInfo>,
    /// The last modification time.
    pub last_modification: Option<LastModificationInfo>,
    /// The unidentified chunks.
    pub unknown_chunks: Vec<UnknownChunkInfo>,
}
