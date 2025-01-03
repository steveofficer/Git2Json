mod git_parser;

use crate::git_parser::pack::read_packfile;

fn main() {
    println!("Pack File");
    //read_packfile("C:/Dev/Git2Json/.git/objects/pack/pack-07c4c5a79def263705060469408d2fad29a450e4.pack");
    read_packfile("C:/Dev/TypeScript/.git/objects/pack/pack-24587a8b72c96b45e6de22a3d4ef78ca305625c4.pack"); 
}
