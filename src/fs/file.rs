use std::io;

pub fn read_stream_u32<R>(mut reader: R) -> Result<Vec<u32>, io::Error>
where
    R: io::Read + io::Seek,
{
    let u32_length = reader.stream_len()? as usize;
    let mut array = Vec::new();
    array.reserve(u32_length);
    let mut tmp = [0_u8; 4];
    while reader.read(&mut tmp)? == 4 {
        array.push(u32::from_le_bytes(tmp));
    }
    Ok(array)
}
