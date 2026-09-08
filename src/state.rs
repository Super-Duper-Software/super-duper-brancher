use crate::time::get_epoch_secs;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize)]
pub struct Branch {
    parent_name: String,
    fork_point: String,
    created_at: u64,
}

impl Branch {
    pub fn new(parent_name: String, fork_point: String) -> Self {
        let now = get_epoch_secs();
        Branch {
            created_at: now,
            fork_point: Branch::format_fork_point(fork_point),
            parent_name: Branch::format_parent_name(parent_name),
        }
    }

    fn format_fork_point(fork_point: String) -> String {
        String::from(fork_point.trim())
    }

    fn format_parent_name(parent_name: String) -> String {
        String::from(parent_name.trim())
    }

    pub fn fork_point(&self) -> &str {
        &self.fork_point
    }

    pub fn parent_name(&self) -> &str {
        &self.parent_name
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }
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
