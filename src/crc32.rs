const CRC32_TABLE: [u32; 256] = make_table();

const fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];

    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut k = 0;
        while k < 8 {
            crc = (crc >> 1) ^ ((crc & 1) * 0xedb88320u32);
            k += 1;
        }

        table[i] = crc;
        i += 1;
    }

    return table;
}

/// Computes the CRC32 checksum for the given data slice.
///
/// # Arguments
/// * `data` - A byte slice to compute the checksum for.
///
/// # Returns
/// * `u32` - The CRC32 checksum.
#[inline]
pub fn compute(data: &[u8]) -> u32 {
    let mut crc = 0xffffffffu32;

    for &byte in data {
        let index = ((crc ^ (byte as u32)) & 0xffu32) as usize;
        crc = CRC32_TABLE[index] ^ (crc >> 8u32);
    }

    return crc ^ 0xffffffffu32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_known_value() {
        let data = b"123456789";
        assert_eq!(compute(data), 0xcbf43926);
    }
}
