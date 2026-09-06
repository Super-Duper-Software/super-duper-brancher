use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute process");
    let output_string = String::from_utf8(output.stdout)
        .expect("Failed to parse command output");
    println!("response: {}", output_string.trim());
}
