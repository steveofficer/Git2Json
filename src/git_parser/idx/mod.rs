use std::io::Error;
use std::{
    str,
    io
};
use nom::bytes::streaming::take;
use nom::combinator::map;
use nom::IResult;
use nom::{
    error::ErrorKind,
    number::streaming::be_u32,
    multi::count
};


use crate::git_parser::types;

use super::types::GitIndexFile;

fn parse_sha(input: &[u8]) -> IResult<&[u8], String> {
    return map(
        take(20usize), |h: &[u8]| String::from_iter(h.iter().map(|b| format!("{:02x?}", b)))
    )(input)
}

pub fn read_idx(path: &str) -> io::Result<types::GitIndexFile> {
    let contents = std::fs::read(path)?;

    let (magic_number, remainder) = contents.split_at(4);
    println!("Magic Number: {:?}", magic_number);

    let (version_number, remainder) = remainder.split_at(4);
    println!("Version Number: {:?}", version_number);

    let (remainder, fanout_result) = 
        match count(be_u32::<_, (_, ErrorKind)>, 256)(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("Fanout: {:?}", fanout_result);

    let object_count: usize = fanout_result[fanout_result.len()-1].try_into().unwrap();
    
    let (remainder, shas) = 
        match count(parse_sha, object_count)(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("SHA Count: {:?}", shas.len());

    let (remainder, crcs) = 
        match count(take::<usize, &[u8], nom::error::Error<_>>(4usize), object_count)(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("Crc Count: {:?}", crcs.len());

    let (remainder, offsets) = 
        match count(be_u32::<_, (_, ErrorKind)>, object_count)(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("Offset Count: {:?}", offsets.len());

    let (remainder, packfile_checksum) = 
        match parse_sha(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("Packfile Hash: {:?}", packfile_checksum);

    let (remainder, idx_checksum) = 
        match parse_sha(remainder) {
            Ok((remainder, t)) => Ok((remainder, t)),
            Err(_e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
        }?;
    println!("Idx Hash: {:?}", idx_checksum);

    println!("Remaining Bytes: {:?}", remainder);

    Ok(GitIndexFile { object_names: shas, offsets: offsets })
}