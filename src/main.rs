use std::process::Command;
use std::process::Output;
use std::str;

fn handle_git_output(output: &Output) {
    if output.status.success() {
        println!("Git output {:?}", str::from_utf8(&output.stdout))
    } else {
        println!("Git error {:?}", str::from_utf8(&output.stderr))
    }
}

fn main() {
    let git_result = Command::new("git")
      .args([ "log", "-1" ])
      .output();
    match git_result {
        | Ok(git_output) => handle_git_output(&git_output),
        | Err(err) => println!("Error {}", err) 
    }
}
