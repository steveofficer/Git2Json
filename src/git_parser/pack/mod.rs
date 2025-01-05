// http://shafiul.github.io/gitbook/7_the_packfile.html
// https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::{BufReader, Read, Seek},
    str,
};

use flate2::read::ZlibDecoder;

use crate::git_parser;

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OffsetDelta = 6,
    ReferenceDelta = 7,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RawGitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    OffsetDelta(u64),
    ReferenceDelta(String),
}

#[derive(Debug, PartialEq)]
pub struct RawGitObject {
    pub object_type: RawGitObjectType,
    pub position: u64,
    pub content: Vec<u8>,
}

fn parse_sha<R: Read>(source: &mut R) -> Result<String, std::io::Error> {
    let mut magic_bytes: [u8; 20] = [0; 20];
    source.read_exact(&mut magic_bytes)?;
    Ok(String::from_iter(
        magic_bytes.iter().map(|b| format!("{:02x?}", b)),
    ))
}

fn compute_git_hash(object_type: &RawGitObjectType, content: &[u8]) -> String {
    let mut hasher = sha1_smol::Sha1::new();

    let object_prefix = match object_type {
        RawGitObjectType::Blob => "blob",
        RawGitObjectType::Commit => "commit",
        RawGitObjectType::Tree => "tree",
        RawGitObjectType::Tag => "tag",
        _ => "",
    };

    hasher.update(format!("{} {}\0", object_prefix, content.len()).as_bytes());
    hasher.update(&content);

    hasher.hexdigest()
}

fn decode<R: Read + Seek>(
    content: &mut R,
    length: usize,
) -> Result<(Vec<u8>, u64), std::io::Error> {
    let mut decoder = ZlibDecoder::new(content);

    let mut decompressed_contents = Vec::with_capacity(length);

    decoder.read_to_end(&mut decompressed_contents)?;
    Ok((decompressed_contents, decoder.total_in()))
}

fn decompress_contents<R: Read + Seek>(
    content: &mut R,
    length: usize,
) -> Result<Vec<u8>, std::io::Error> {
    let start_position = content.stream_position()?;
    let (decoded_content, read_bytes) = decode(content, length)?;

    content.seek(std::io::SeekFrom::Start(start_position + read_bytes))?;
    Ok(decoded_content)
}

fn read_object_header<R: Read + Seek>(
    input: &mut R,
) -> Result<(ObjectType, usize), std::io::Error> {
    let mut buf = [0];
    input.read(&mut buf)?;
    let mut read_next = (buf[0] & 0b1000_0000) == 0b1000_0000;

    let object_type = match (buf[0] & 0b0111_0000) >> 4 {
        1 => ObjectType::Commit,
        2 => ObjectType::Tree,
        3 => ObjectType::Blob,
        4 => ObjectType::Tag,
        6 => ObjectType::OffsetDelta,
        7 => ObjectType::ReferenceDelta,
        _ => panic!("Invalid object type"),
    };

    let mut length = u32::from_be_bytes([0, 0, 0, buf[0] & 0b0000_1111]);

    let mut offset = 1;
    while read_next {
        input.read(&mut buf)?;
        read_next = (buf[0] & 0b1000_0000) == 0b1000_0000;
        let length_part = u32::from_be_bytes([0, 0, 0, buf[0] & 0b0111_1111]);

        length = (length_part << (offset - 1) * 7 + 4) | length;
        offset += 1;
    }

    Ok((object_type, length.try_into().unwrap()))
}

fn read_object_entry<R: Read + Seek>(input: &mut R) -> Result<RawGitObject, std::io::Error> {
    let position = input.stream_position()?;
    let (object_type, uncompressed_length) = read_object_header(input)?;

    match object_type {
        ObjectType::ReferenceDelta => {
            let sha = parse_sha(input)?;
            Ok(RawGitObject {
                object_type: RawGitObjectType::ReferenceDelta(sha),
                position,
                content: vec![],
            })
        }

        ObjectType::OffsetDelta => {
            let mut ref_offset = 0;
            let mut read_next = true;
            let mut buf = [0];

            // Each byte contributes 7 bits of data
            const VARINT_ENCODING_BITS: u8 = 7;
            // The upper bit indicates whether there are more bytes
            const VARINT_CONTINUE_FLAG: u8 = 1 << VARINT_ENCODING_BITS;

            while read_next {
                input.read(&mut buf)?;

                read_next = (buf[0] & VARINT_CONTINUE_FLAG) != 0;

                ref_offset = (ref_offset << 7) | (buf[0] & 0b0111_1111) as u64;

                if read_next {
                    ref_offset += 1;
                }
            }

            let content = decompress_contents(input, uncompressed_length)?;

            Ok(RawGitObject {
                object_type: RawGitObjectType::OffsetDelta(
                    position.checked_sub(ref_offset).unwrap(),
                ),
                position,
                content,
            })
        }

        _ => {
            let content = decompress_contents(input, uncompressed_length)?;

            let object_type = match object_type {
                ObjectType::Blob => RawGitObjectType::Blob,
                ObjectType::Commit => RawGitObjectType::Commit,
                ObjectType::Tag => RawGitObjectType::Tag,
                ObjectType::Tree => RawGitObjectType::Tree,
                _ => panic!("Unknown object type"),
            };

            Ok(RawGitObject {
                object_type,
                position,
                content,
            })
        }
    }
}

pub fn read_packfile(path: &str) -> Result<(), std::io::Error> {
    // Read the contents of the pack file
    let raw_stream = std::fs::File::open(path)?;
    let mut pack_stream = BufReader::new(raw_stream);

    // Read PACK
    let mut byte_buffer = [0, 0, 0, 0];
    pack_stream.read_exact(&mut byte_buffer)?;
    println!("PACK: {:?}", str::from_utf8(&byte_buffer));

    // Read the packfile version
    pack_stream.read_exact(&mut byte_buffer)?;
    println!("Version: {:?}", byte_buffer);

    // Read the number of objects that will appear in the packfile
    pack_stream.read_exact(&mut byte_buffer)?;
    let object_count = u32::from_be_bytes(byte_buffer);
    println!("Object Count: {:?}", object_count);

    let mut hash_positions: HashMap<String, u64> = HashMap::new();
    let mut positions: HashSet<u64> = HashSet::new();

    // Loop over object_count objects
    for _ in 0..object_count {
        let RawGitObject {
            object_type,
            position,
            content,
        } = read_object_entry(&mut pack_stream)?;

        let object_hash = compute_git_hash(&object_type, &content);

        println!(
            "Type: {:?}  Position: {} Hash: {}",
            object_type, position, object_hash
        );

        if object_type == RawGitObjectType::Commit {
            let (_, commit_entry) = git_parser::commit::parse_git_commit_entry(&content)
                .expect("Could not read commit entry");
            println!("{:?}", commit_entry);
            println!("END OF CONTENT");
        } else if let RawGitObjectType::OffsetDelta(offset) = object_type {
            match positions.contains(&offset) {
                true => {
                    println!("Found an object at position {offset}");
                }
                false => {
                    println!("Did not find an object at position {offset}");
                }
            }
        }
        hash_positions.insert(object_hash, position);
        positions.insert(position);
    }
    Ok(())
}
