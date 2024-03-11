mod git_parser;

use std::io;
use std::fs::read_to_string;

use std::str;

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

fn main() {
    let head_commit = find_head_ref("C:/Dev/Git2Json");
    let head_content = head_commit.expect("Failed to read starting commit");
    println!("Starting with commit {:?}", head_content);

    let (folder, path) = hash_to_file_path(&head_content);
    let head_file_path = format!("C:/Dev/Git2Json/.git/objects/{}/{}", folder, path);
    println!("Reading {}", head_file_path);
    
    let content = git_parser::read_git_file(&head_file_path);
    println!("HEAD commit content\n{:?}", content);

    let (folder, path) = hash_to_file_path("18ca9cadb6d1383000eb95ef956f8aa7c99c1680");
    let file_path = format!("C:/Dev/Git2Json/.git/objects/{}/{}", folder, path);
    println!("{}", file_path);
    println!("{:?}", git_parser::read_git_file(&file_path));

}
