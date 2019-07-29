//! File reading/writing.

use std::io;
use std::io::{Read, Seek};

/// Read from a readable, seekable stream into an [`Vec<u8>`](Vec).
///
/// Returns a result with an [`io::Error`](std::io::Error) if there is an issue reading.
/// Wrapper around [`read_to_end`](std::io::Read::read_to_end).
///
/// # Example
///
/// ```edition2018,no_run
/// # use nova_rs::fs::file::read_stream_u8;
/// let mut file = std::fs::File::open("my_file")?;
/// let result: Vec<u8> = read_stream_u8(&mut file)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn read_stream_u8<R>(mut reader: R) -> Result<Vec<u8>, io::Error>
where
    R: io::Read + io::Seek,
{
    let mut array = Vec::new();
    reader.read_to_end(&mut array)?;
    Ok(array)
}

/// Read from a readable, seekable stream into an [`Vec<u32>`](Vec).
///
/// Returns a result with an [`io::Error`](std::io::Error) if there is an issue reading. Uses a
/// [`io::BufReader`](std::io::BufReader) internally due to needing many 4 byte reads.
///
/// # Example
///
/// ```edition2018,no_run
/// # use nova_rs::fs::file::read_stream_u32;
/// let mut file = std::fs::File::open("my_file")?;
/// let result: Vec<u32> = read_stream_u32(&mut file)?;
/// # Ok::<(), std::io::Error>(())
/// ```
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

/// Read from a readable, seekable stream into an [`String`].
///
/// Returns a result with an [`io::Error`](std::io::Error) if there is an issue reading.
/// Wrapper around [`read_to_string`](std::io::Read::read_to_string).
///
/// # Example
///
/// ```edition2018,no_run
/// # use nova_rs::fs::file::read_stream_string;
/// let mut file = std::fs::File::open("my_file")?;
/// let result: String = read_stream_string(&mut file)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn read_stream_string<R>(mut reader: R) -> Result<String, io::Error>
where
    R: io::Read + io::Seek,
{
    let mut string = String::new();
    reader.read_to_string(&mut string)?;
    Ok(string)
}
