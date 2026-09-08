use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize)]
pub struct Branch {
    pub parent_name: String,
    pub fork_point: String,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub branches: HashMap<String, Branch>,
    pub installed_at: u64,
}

pub fn read_state() -> Result<State> {
    let state_file = match fs::read_to_string(".git/sdb-state.json") {
        Ok(file_string) => file_string,
        Err(_error) => {
            return Ok(State {
                branches: HashMap::new(),
                installed_at: 0,
            });
        }
    };
    serde_json::from_str(&state_file)
}

pub fn write_state(state: &State) {
    let state_string = serde_json::to_string_pretty(state)
        .expect("Error converting in-memory state to pretty string");
    fs::write(".git/sdb-state.json", state_string).expect("Error writing in-memory state to file")
}
