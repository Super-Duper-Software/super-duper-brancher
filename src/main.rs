use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

// Branch state

#[derive(Serialize, Deserialize)]
struct Branch {
    parent_name: String,
    created_at: u64,
}

#[derive(Serialize, Deserialize)]
struct State {
    branches: HashMap<String, Branch>,
    installed_at: u64,
}

fn read_state() -> Result<State> {
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

fn write_state(state: &State) {
    let state_string = serde_json::to_string_pretty(state)
        .expect("Error converting in-memory state to pretty string");
    fs::write(".git/sdb-state.json", state_string).expect("Error writing in-memory state to file")
}

// CLI
#[derive(Parser)]
#[command(name = "SuperDuperBrancher")]
#[command(version = "1.0")]
#[command(about = "Does awesome things", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Status,
    HookPostCheckout(HookPostCheckoutArgs),
    HookPostMerge(HookPostMergeArgs),
}

#[derive(Args)]
struct HookPostCheckoutArgs {
    ref_prev_head: String,
    ref_new_head: String,
    /// 0 for file checkout, 1 for branch checkout
    is_branch_checkout: i32,
}

#[derive(Args)]

struct HookPostMergeArgs {
    /// 0 for standard, 1 for squash
    is_squash: i32,
}

fn get_script(subcommand: &str) -> String {
    let exe_path = match env::current_exe() {
        Ok(current_exe_path) => {
            if let Some(path_str) = current_exe_path.to_str() {
                path_str.to_string()
            } else {
                panic!("Could not parse current exe path")
            }
        }
        Err(e) => panic!("failed to get current exe path: {e}"),
    };

    format!("#!/bin/sh\nexec {exe_path} {subcommand} \"$@\"\n")
}

fn write_and_set_perms(path: &str, script: &String) {
    fs::write(path, script).expect("could not write file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .expect("Failed to set file permissions");
    println!("Wrote script to {}", path);
}

fn get_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock before epoch")
        .as_secs()
}

fn init() {
    let post_checkout_script = get_script("hook-post-checkout");
    let post_merge_script = get_script("hook-post-merge");

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);
    write_and_set_perms(".git/hooks/post-merge", &post_merge_script);

    let mut state = read_state().expect("Could not read state");
    state.installed_at = get_epoch_secs();
    write_state(&state);
}

fn status() {
    println!("You invoked the status command!");
}

fn hook_post_checkout(_prev: &str, _new: &str, is_branch_checkout: &i32) {
    if *is_branch_checkout == 0 {
        return;
    }

    let current_branch_name = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute rev-parse");

    let current_branch_name_string =
        String::from_utf8(current_branch_name.stdout).expect("Error getting string from output");

    let mut state = read_state().expect("Error reading state");
    if state
        .branches
        .contains_key(current_branch_name_string.trim())
    {
        return;
    }

    let reflog_time_output = Command::new("git")
        .args([
            "log",
            "-g",
            "--format=%gd",
            "--date=unix",
            current_branch_name_string.trim(),
        ])
        .output()
        .expect("failed to get list of reflog timestamps");
    let reflog_time_output: String =
        String::from_utf8(reflog_time_output.stdout).expect("Failed to parse output");
    let last_reflog_time = reflog_time_output.lines().last();
    if let Some(last_reflog) = last_reflog_time {
        let last_reflog = last_reflog
            .trim()
            .split_once('{')
            .expect("could not split")
            .1;
        let last_reflog_digits = last_reflog.split_once('}').expect("could not split").0;

        let last_reflog_digits: u64 = last_reflog_digits
            .trim()
            .parse()
            .expect("Could not convert reflog to timestamp");

        if last_reflog_digits < state.installed_at {
            return;
        }
    } else {
        panic!("Could not parse last reflog time")
    }

    let previous_branch_name = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "@{-1}"])
        .output()
        .expect("Failed to execute rev-parse");

    let previous_branch_name_string =
        String::from_utf8(previous_branch_name.stdout).expect("Error getting string from output");

    if previous_branch_name_string.is_empty() {
        return;
    }

    let now = get_epoch_secs();

    let current_branch = Branch {
        created_at: now,
        parent_name: String::from(previous_branch_name_string.trim()),
    };

    state.branches.insert(
        String::from(current_branch_name_string.trim()),
        current_branch,
    );

    write_state(&state);
}

fn hook_post_merge(is_squash: &i32) {
    println!(
        "You invoked the hook_post_merge command. is squash {}",
        is_squash
    );
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => init(),
        Commands::Status => status(),
        Commands::HookPostCheckout(HookPostCheckoutArgs {
            ref_prev_head,
            ref_new_head,
            is_branch_checkout,
        }) => {
            hook_post_checkout(ref_prev_head, ref_new_head, is_branch_checkout);
        }
        Commands::HookPostMerge(HookPostMergeArgs { is_squash }) => {
            hook_post_merge(is_squash);
        }
    }
}
