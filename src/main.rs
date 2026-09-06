use std::{collections::HashMap, fs, process::Command, time::{SystemTime, UNIX_EPOCH}};
use serde::{Serialize, Deserialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct Branch {
    parent_name: String,
    created_at: u64
}

#[derive(Serialize, Deserialize)]
struct State {
    branches: HashMap<String, Branch>
}

fn read_state() -> Result<State> {
    let state_file = match fs::read_to_string(".git/sdb-state.json") {
        Ok(file_string) => file_string,
        Err(error) => {
            println!("No state file for sdb: {}", error);
            return Ok(State {
                branches: HashMap::new()
            });
        }
    };
    serde_json::from_str(&state_file)
}

fn write_state(state: &State) {
    let state_string = serde_json::to_string_pretty(state)
        .expect("Error converting in-memory state to pretty string");
    fs::write(".git/sdb-state.json", state_string)
        .expect("Error writing in-memory state to file")
}

fn main() {
    let mut state = read_state()
        .expect("Error reading state");
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute process");
    let output_string: String = String::from_utf8(output.stdout)
        .expect("Error getting string from output");

    let now = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Clock before epoch");
        

    let current_branch = Branch {
        created_at: now.as_secs(),
        parent_name: String::from("")
    };

    state.branches.insert(String::from(output_string.trim()), current_branch);
    write_state(&state);
}
