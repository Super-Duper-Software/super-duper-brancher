use std::ffi::OsStr;
use std::process::Command;

pub fn run_git<I, S>(commands: I) -> (String, Option<i32>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(commands)
        .output()
        .expect("Failed to run git command");
    let stdout = String::from_utf8(output.stdout).expect("Error getting string from git output");
    let code = output.status.code();
    (stdout, code)
}

pub fn is_ancestor(child: &str, ancestor: &str) -> bool {
    let (_, code) = run_git(["merge-base", "--is-ancestor", child, ancestor]);
    code == Some(0)
}

pub fn current_branch() -> Option<String> {
    let (out, code) = run_git(["branch", "--show-current"]);
    if code != Some(0) {
        return None;
    }
    let name = out.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
