use std::process::Command;
use std::str;

fn main() {
    let output = Command::new("git")
      .args([ "log", "-1" ])
      .output();
    match output {
        | Ok(x) => println!("Ok {:?}", str::from_utf8(&x.stderr)),
        | Err(e) => println!("Error {}", e) 
    }
}
