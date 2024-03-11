use std::io::{prelude::*, Error};
use flate2::read::ZlibDecoder;
use std::{
    str,
    io
};

use nom::{
    branch::alt, bytes::{complete::is_not, streaming::{tag, take}}, character::{complete::{digit1, multispace0, one_of, space1}, is_newline, streaming::{alphanumeric1, char}}, combinator::{map, recognize, rest, rest_len, value}, multi::{many0, many1_count, many_m_n, many_till, separated_list0}, sequence::{delimited, preceded, terminated, tuple}, IResult
};

#[derive(Clone, Debug)]
pub enum ObjectType {
    Tree,
    Commit,
    Blob
}

#[derive(Clone, Debug)]
pub struct GitTreeEntry {
    code: String,
    file_name: String,
    hash: String
}

#[derive(Clone, Debug)]
pub struct GitTree {
    entries: Vec<GitTreeEntry>
}

#[derive(Clone, Debug)]
pub struct GitCommit {
    metadata: Vec<String>,
    content: Vec<String>
}

#[derive(Clone, Debug)]
pub struct GitBlob {
    content: String
}

#[derive(Clone, Debug)]
pub enum GitObject {
    Commit(GitCommit),
    Tree(GitTree),
    Blob(GitBlob)
}

fn parse_object_type(input: &[u8]) -> IResult<&[u8], ObjectType> {
    alt((
        value(ObjectType::Blob, tag("blob")),
        value(ObjectType::Commit, tag("commit")),
        value(ObjectType::Tree, tag("tree")),
    ))(input)
}

fn parse_object_length(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(digit1, char('\0'))(input)
}

fn parse_tree_entry(input: &[u8]) -> IResult<&[u8], GitTreeEntry> {
    let (remainder, code) = terminated(digit1, space1)(input)?;
    let code_str = str::from_utf8(code).expect("Could not parse tree entry code");
    
    let (remainder, file_name) = terminated(is_not("\0"), char('\0'))(remainder)?;
    let file_name_str = str::from_utf8(file_name).expect("Could not parse tree entry file name");
    
    let (remainder, hash) = map(
        take(20usize), |h: &[u8]| String::from_iter(h.iter().map(|b| format!("{:02x?}", b)))
    )(remainder)?;
    
    Ok((
        remainder, 
        GitTreeEntry { 
            code: code_str.to_string(),
            file_name: file_name_str.to_string(),
            hash: hash
        }
    ))
}

fn parse_lines(input: &[u8]) -> IResult<&[u8], Vec<&str>> {
    separated_list0(
        one_of("\r\n"), 
        map(is_not("\r\n"), |x| str::from_utf8(x).expect(""))
    )(input)
}

fn parse_git_object(input: &[u8]) -> IResult<&[u8], GitObject> {
    let (remainder, (object_info, _)) = 
        tuple((
            parse_object_type,
            preceded(multispace0, parse_object_length),
        ))(input)?;

    match object_info {
        ObjectType::Tree => {
            let (remainder, result) = many0(parse_tree_entry)(remainder)?;
            Ok((remainder, GitObject::Tree(GitTree { entries: result })))
        }

        ObjectType::Commit => {
            let (remainder, metadata_result) = parse_lines(remainder)?;

            let (remainder, content_result) = preceded(many0(one_of("\r\n")), parse_lines)(remainder)?;

            Ok((
                remainder,
                GitObject::Commit(GitCommit {
                    metadata: metadata_result.iter().map(|x| x.to_string()).collect(),
                    content: content_result.iter().map(|x| x.to_string()).collect(),
                })
            ))
        }
        
        ObjectType::Blob => {
            let (remainder, result) = map(rest, |d| str::from_utf8(d).expect(""))(remainder)?;
            Ok((remainder, GitObject::Blob(GitBlob { content: result.to_string() })))
        }
    }
}

pub fn read_git_file(path: &str) -> io::Result<String> {
    let contents = std::fs::read(path)?;
    let mut decoder = ZlibDecoder::new(&contents[..]);
    
    let mut decompressed_contents = Vec::new();
    decoder.read_to_end(&mut decompressed_contents)?;
    
    //Ok(decompressed_contents)
    let result = parse_git_object(&decompressed_contents);
    match result {
        Ok((r, t)) => {
            println!("{:?}", t);
            println!("{:?}", r);
            Ok("Hello".to_string())
        },
        Err(e) => io::Result::Err(Error::new(io::ErrorKind::InvalidData, ""))
    }
}