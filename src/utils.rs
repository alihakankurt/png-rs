use std::io::{Read, Seek, SeekFrom};

use crate::error::ParserError;

/// Converts the provided slice of data to vector using the provided projection function.
pub fn to_chunked<const N: usize, F, To>(slice: &[u8], f: F) -> Vec<To>
where
    F: FnMut(&[u8; N]) -> To,
{
    return slice.as_chunks::<N>().0.iter().map(f).collect();
}

/// Converts the provided slice of data to unsigned 16-bit integer.
pub fn to_u16(slice: &[u8]) -> u16 {
    debug_assert_eq!(slice.len(), 2);
    return u16::from_be_bytes([slice[0], slice[1]]);
}

/// Converts the provided slice of data to unsigned 32-bit integer.
pub fn to_u32(slice: &[u8]) -> u32 {
    debug_assert_eq!(slice.len(), 4);
    return u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
}

/// Converts the provided slice of data to 32-bit floating point.
pub fn to_f32(slice: &[u8]) -> f32 {
    return to_u32(slice) as f32 / 100000.0f32;
}

/// Converts the provided slice of data to a owned string.
pub fn to_string(slice: &[u8]) -> String {
    return String::from_utf8(slice.to_vec()).unwrap();
}

/// Tries to get first null-terminated string out of the provided slice of data.
pub fn get_string(slice: &[u8]) -> Result<String, ParserError> {
    let terminator = match slice.iter().position(|&b| b == 0) {
        Some(index) => index,
        None => {
            return Err(ParserError::MissingNullTerminator);
        }
    };

    return Ok(to_string(&slice[..terminator]));
}

/// Checks whether the provided string has length of valid range according to the PNG specification.
pub fn validate_string(s: &str) -> Result<(), ParserError> {
    if s.is_empty() || s.len() > 79 {
        return Err(ParserError::InvalidStringLength);
    }

    return Ok(());
}

/// Reads next 4 bytes from the provided source and converts it to an unsigned 32-bit integer.
pub fn read_u32<Source: Read>(source: &mut Source) -> Result<u32, ParserError> {
    let mut buffer = [0u8; 4];
    match source.read_exact(&mut buffer) {
        Ok(()) => Ok(to_u32(&buffer)),
        Err(e) => Err(ParserError::IOError(e)),
    }
}

/// Reads next n bytes from the provided source into the provided buffer.
pub fn read_to<'a, Source: Read>(
    source: &mut Source,
    buffer: &'a mut [u8],
) -> Result<&'a [u8], ParserError> {
    match source.read_exact(buffer) {
        Ok(()) => Ok(&buffer[..]),
        Err(e) => Err(ParserError::IOError(e)),
    }
}

/// Reads next n bytes from the provided source and returns a vector of bytes.
pub fn read_bytes<Source: Read>(source: &mut Source, size: usize) -> Result<Vec<u8>, ParserError> {
    let mut bytes = vec![0u8; size];
    match source.read_exact(&mut bytes) {
        Ok(()) => Ok(bytes),
        Err(e) => Err(ParserError::IOError(e)),
    }
}

/// Seeks the position of the provided source.
pub fn seek<Source: Seek>(source: &mut Source, pos: i64) -> Result<(), ParserError> {
    match source.seek(SeekFrom::Current(pos)) {
        Ok(_) => Ok(()),
        Err(e) => Err(ParserError::IOError(e)),
    }
}
