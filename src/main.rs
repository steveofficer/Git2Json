mod git_parser;

use std::collections::VecDeque;
use std::io;
use std::fs::read_to_string;

use std::str;

use crate::git_parser::pack::read_pack;
use crate::git_parser::idx::read_idx;
use crate::git_parser::read_git_file;
use crate::git_parser::types::GitObject;

fn find_head_ref(directory_name: &str) -> io::Result<String> {
    // The content .git/HEAD of points us to a file that contains the HEAD commit.
    let head_content= read_to_string(format!("{}/.git/HEAD", directory_name))?;
    // Drop the leading "ref: " and get the actual file name 
    let head_ref = &head_content[5..];

    // Get the commit from that file
    let full_ref = format!("{}/.git/{}", directory_name, head_ref.trim_end());
    
    let head_commit = read_to_string(full_ref)?;
    
    Ok(String::from(head_commit.trim_end()))
}

fn hash_to_file_path(hash: &str) -> (&str, &str) {
    (&hash[..2], &hash[2..])
}

fn process_git_object(o: &GitObject, hashes: &mut VecDeque<String>) {
    match o {
        GitObject::Tree(tree) => {
            for ele in &tree.entries {
                hashes.push_back(ele.hash.to_string())
            }
        }

        GitObject::Commit(commit) => {
            for data in &commit.metadata {
                if data.starts_with("tree") {
                    let hash = data[5..].to_string();
                    println!("Discovered {}", hash);
                    hashes.push_back(hash)
                } else if data.starts_with("parent") {
                    let hash = data[7..].to_string();
                    println!("Discovered {}", hash);
                    hashes.push_back(hash)
                }
            }
        }

        _ => {}
    }
}



fn main() {
    println!("Pack File");
    read_pack("C:/Dev/Git2Json/.git/objects/pack/pack-07c4c5a79def263705060469408d2fad29a450e4.pack");
    
    println!("Idx File");
    let _idx_file = read_idx("C:/Dev/Git2Json/.git/objects/pack/pack-07c4c5a79def263705060469408d2fad29a450e4.idx");
    //println!("{:?}", idx_file);
    
    println!("Git Objects");

    let working_dir = "C:/Dev/Git2Json";
    let head_commit = find_head_ref(working_dir);
    let head_content = head_commit.expect("Failed to read HEAD commit");
    println!("Starting with commit {:?}", head_content);

    let mut object_hashes = VecDeque::new();
    object_hashes.push_back(head_content);
    
    while object_hashes.len() > 0 {
        let hash = object_hashes.pop_front().expect("");
        
        let (folder, path) = hash_to_file_path(&hash);
        let file_path = format!("{}/.git/objects/{}/{}", working_dir, folder, path);
        println!("Parsing {}", file_path);

        let o = read_git_file(&file_path).expect("");
        process_git_object(&o, &mut object_hashes);
    }
}
