// http://shafiul.github.io/gitbook/7_the_packfile.html

use std::io::{prelude::*, Error};
use std::{
    str,
    io
};

use nom::{
    error::ErrorKind,
    number::streaming::be_u32
};

pub fn read_pack(path: &str) {
    //C:\Dev\dapperdox\.git\objects\pack
    let contents = std::fs::read(path).expect("");
    let (prelude, remainder) = contents.split_at(4);
    println!("{:?}", str::from_utf8(prelude));

    let (version, remainder) = remainder.split_at(4);
    println!("{:?}", version);

    let count_result = be_u32::<_, (_, ErrorKind)>(remainder);
    let (remainder, count) = count_result.expect("Count");
    println!("{:?}", count);

    println!("{:?}", remainder[0]);

}