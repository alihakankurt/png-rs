use std::{
    fmt::{Display, Formatter, Result},
    io::Error,
};

/// Represents the errors related to the parser.
#[derive(Debug)]
pub enum ParserError {
    IOError(Error),
    InvalidSignature,
    InvalidChunkLength(u32),
    InvalidChunkOrder(u32),
    DuplicateChunk(u32),
    MissingRequiredChunk(u32),
    InvalidFieldValue,
    NonConsecutiveData,
    MissingNullTerminator,
    InvalidStringLength,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ParserError::IOError(e) => write!(f, "IO error: {}", e),
            ParserError::InvalidSignature => write!(f, "Invalid PNG signature"),
            ParserError::InvalidChunkLength(chunk_id) => write!(
                f,
                "{} chunk has invalid chunk length",
                str::from_utf8(&u32::to_be_bytes(*chunk_id)).unwrap()
            ),
            ParserError::InvalidChunkOrder(chunk_id) => {
                write!(
                    f,
                    "Order of {} chunk is invalid for PNG specification",
                    str::from_utf8(&u32::to_be_bytes(*chunk_id)).unwrap()
                )
            }
            ParserError::DuplicateChunk(chunk_id) => write!(
                f,
                "Multiple occurences of {} chunk found, it should exist only once",
                str::from_utf8(&u32::to_be_bytes(*chunk_id)).unwrap()
            ),
            ParserError::InvalidFieldValue => write!(f, "A field contains out of range value"),
            ParserError::NonConsecutiveData => write!(f, "IDAT chunks are not consecutive"),
            ParserError::MissingNullTerminator => {
                write!(f, "Missing null-terminator for character string")
            }
            ParserError::InvalidStringLength => {
                write!(
                    f,
                    "Character strings like keyword/name must have a length between 1-79 inclusive"
                )
            }
            ParserError::MissingRequiredChunk(chunk_id) => write!(
                f,
                "Chould not be able to find {} chunk which is required",
                str::from_utf8(&u32::to_be_bytes(*chunk_id)).unwrap()
            ),
        }
    }
}
