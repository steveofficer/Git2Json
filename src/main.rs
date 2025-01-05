mod git_parser;

use crate::git_parser::pack::read_packfile;

fn main() {
    println!("Pack File");
    //let pack_file_path = "C:/Dev/Git2Json/.git/objects/pack/pack-07c4c5a79def263705060469408d2fad29a450e4.pack";
    let pack_file_path =
        "C:/Dev/TypeScript/.git/objects/pack/pack-24587a8b72c96b45e6de22a3d4ef78ca305625c4.pack";
    let _ = read_packfile(pack_file_path);
}
