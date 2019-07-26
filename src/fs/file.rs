use std::fs;
use std::io;
use std::io::{Read, Seek};

pub fn read_stream_u8<R>(mut reader: R) -> Result<Vec<u8>, io::Error>
where
    R: io::Read + io::Seek,
{
    let mut buffered_reader = io::BufReader::new(&mut reader);
    let mut array = Vec::new();
    buffered_reader.read_to_end(&mut array)?;
    Ok(array)
}

pub fn read_stream_u32<R>(mut reader: R) -> Result<Vec<u32>, io::Error>
where
    R: io::Read + io::Seek,
{
    let mut buffered_reader = io::BufReader::new(&mut reader);

    let mut array = Vec::new();
    let length = (buffered_reader.stream_len()? as usize) / 4;
    array.reserve(length);

    let mut tmp = [0_u8; 4];
    while buffered_reader.read(&mut tmp)? == 4 {
        array.push(u32::from_le_bytes(tmp));
    }

    Ok(array)
}

pub fn read_stream_string<R>(mut reader: R) -> Result<String, io::Error>
where
    R: io::Read + io::Seek,
{
    let mut buffered_reader = io::BufReader::new(&mut reader);
    let mut string = String::new();
    buffered_reader.read_to_string(&mut string)?;
    Ok(string)
}
