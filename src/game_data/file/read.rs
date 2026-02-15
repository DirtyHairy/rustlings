use anyhow::{Result, anyhow};

pub fn read_byte<T: From<u8>>(buffer: &[u8], offset: usize) -> Result<(T, usize)> {
    Ok((
        T::from(
            *buffer
                .get(offset)
                .ok_or(anyhow!("offset {} out of bounds", offset))?,
        ),
        offset + 1,
    ))
}

pub fn read_word_be<T: From<u16>>(buffer: &[u8], offset: usize) -> Result<(T, usize)> {
    Ok((
        T::from(
            read_byte::<u16>(buffer, offset)?.0 | (read_byte::<u16>(buffer, offset + 1)?.0) << 8,
        ),
        offset + 2,
    ))
}

pub fn read_word_le<T: From<u16>>(buffer: &[u8], offset: usize) -> Result<(T, usize)> {
    Ok((
        T::from(
            read_byte::<u16>(buffer, offset + 1)?.0 | (read_byte::<u16>(buffer, offset)?.0) << 8,
        ),
        offset + 2,
    ))
}
