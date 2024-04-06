// http://shafiul.github.io/gitbook/7_the_packfile.html
// https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8
use std::{fmt::Debug, fs::read, io::Read, str};

use nom::{
    bytes::streaming::take, combinator::map, error::ErrorKind, number::{complete::be_i32, streaming::be_u32}, IResult
};

use flate2::read::ZlibDecoder;

fn parse_sha(input: &[u8]) -> IResult<&[u8], String> {
    return map(
        take(20usize), |h: &[u8]| String::from_iter(h.iter().map(|b| format!("{:02x?}", b)))
    )(input)
}

fn read_object_header(input: &[u8]) -> (u8, usize, &[u8]) {
    let mut read_next = (input[0] & 0b1000_0000) == 0b1000_0000;
    println!("Read Next: {}", read_next);
    
    let object_type = (input[0] & 0b0111_0000) >> 4;
    println!("Object Type: {:b}", object_type);

    let mut length = u32::from_be_bytes([0,0,0, input[0] & 0b0000_1111]);
    
    let mut offset = 1;
    while read_next {
        let data = input[offset];
        read_next = (data & 0b1000_0000) == 0b1000_0000;
        let length_part = u32::from_be_bytes([0,0,0,data & 0b0111_1111]);
        println!("Length: {:b}  Length Part: {:b}", length, length_part);

        length = (length_part << (offset-1) * 7 + 4) | length;
        offset += 1;
    }

    let mut output_stream = &input[offset..];

    (object_type, length.try_into().unwrap(), &output_stream)
}

fn decompress_contents(content: &[u8], length: usize) -> (Vec<u8>, usize) {
    let mut decoder = ZlibDecoder::new(content);
    
    let mut decompressed_contents = Vec::with_capacity(length);
    
    let _ = decoder.read_to_end(&mut decompressed_contents);

    (decompressed_contents, decoder.total_in().try_into().unwrap())
}

fn read_object_entry(input: &[u8]) -> &[u8] {
    let (object_type, uncompressed_length, remainder) = read_object_header(input);
    println!("Object Type: {:b}  Object Length: {}", object_type, uncompressed_length);

    if object_type == 7 {
        let (remainder, sha) = parse_sha(remainder).expect("");
        println!("Referenced Object: {}", sha);
        let (_data, remainder) = remainder.split_at(uncompressed_length);
        &remainder
    }
    else if object_type == 6 {
        let mut ref_offset = 0;
        let mut read_next = true;
        let mut offset = 0;
        while read_next {
            let data = remainder[offset];
            read_next = (data & 0b1000_0000) == 0b1000_0000;
            let ref_offset_part = i32::from_be_bytes([0,0,0,data & 0b0111_1111]);
            println!("Offset: {:b}  Offset Part: {:b}", ref_offset, ref_offset_part);

            ref_offset = (ref_offset << 7) | ref_offset_part;
            offset += 1;
            println!("Offset after: {:b}", ref_offset);
        }
        println!("Offset: {}", ref_offset);

        let remaining_bytes = &remainder[offset..];
        let (content, read_bytes) = decompress_contents(remaining_bytes, uncompressed_length);
        println!("Ref content: {:?}", content);

        &remaining_bytes[read_bytes..]
    } else {
        let (decompressed_contents, rl) = decompress_contents(remainder, uncompressed_length);
    
        let body = String::from_utf8_lossy(&decompressed_contents);
        println!("Contents: {:?}", body);
    
        &remainder[rl..]
    }
}

pub fn read_pack(path: &str) {
    //C:\Dev\dapperdox\.git\objects\pack
    let contents = std::fs::read(path).expect("");
    let (prelude, remainder) = contents.split_at(4);
    println!("PACK: {:?}", str::from_utf8(prelude));

    let (version, remainder) = remainder.split_at(4);
    println!("Version: {:?}", version);

    let count_result = be_u32::<_, (_, ErrorKind)>(remainder);
    let (remainder, count) = count_result.expect("Count");
    println!("Object Count: {:?}", count);

    let mut remainder = remainder;

    for _ in 0..count {
        remainder = read_object_entry(remainder);
    }
}