use std::io::{prelude::*, Error};
use flate2::read::ZlibDecoder;
use std::{
    str,
    io
};

mod parsers;
pub mod types;
pub mod idx;
pub mod pack;

pub fn read_git_file(path: &str) -> io::Result<types::GitObject> {
    let contents = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&contents[..]);
    
    let mut decompressed_contents = Vec::new();
    decoder.read_to_end(&mut decompressed_contents)?;
    
    match parsers::parse_git_object(&decompressed_contents) {
        Ok((_, t)) => Ok(t),
        Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
    }
}