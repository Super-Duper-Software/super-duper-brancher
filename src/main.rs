use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::env;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tabled::{Table, Tabled};

// Branch state

#[derive(Serialize, Deserialize)]
struct Branch {
    parent_name: String,
    fork_point: String,
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
}

#[derive(Args)]
struct HookPostCheckoutArgs {
    ref_prev_head: String,
    ref_new_head: String,
    /// 0 for file checkout, 1 for branch checkout
    is_branch_checkout: i32,
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

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);

    let mut state = read_state().expect("Could not read state");
    state.installed_at = get_epoch_secs();
    write_state(&state);
}

fn is_ancestor(child: &str, ancestor: &str) -> bool {
    let (_, code) = run_git(["merge-base", "--is-ancestor", child, ancestor]);
    code == Some(0)
}

#[derive(Tabled)]
struct StatusTable {
    branch: String,
    target: String,
    parent: String,
    is_merged: String,
}

fn get_target(branch_name: &str, state: &State) -> String {
    let branch = state
        .branches
        .get(branch_name)
        .expect("could not get branch in state");
    let mut ancestor_names: Vec<String> = Vec::new();
    let mut current_parent_name = &branch.parent_name;
    let mut current_parent = state.branches.get(&branch.parent_name);

    while current_parent.is_some() {
        match current_parent {
            Some(b) => {
                ancestor_names.push(String::from(current_parent_name));
                current_parent_name = &b.parent_name;
                current_parent = state.branches.get(&b.parent_name);
            }
            None => break,
        }
    }
    ancestor_names.push(String::from(current_parent_name));

    for (index, ancestor) in ancestor_names.iter().enumerate() {
        let mut merged_out = false;
        let ancestor_branch = state.branches.get(ancestor);
        match ancestor_branch {
            Some(ancestor_branch) => {
                let (tip, _) = run_git(["rev-parse", ancestor]);
                if tip.trim() == ancestor_branch.fork_point {
                    return String::from(ancestor);
                }
                for later in ancestor_names[index + 1..].iter() {
                    if is_ancestor(ancestor, later) {
                        merged_out = true;
                        break;
                    }
                }
                if !merged_out {
                    return String::from(ancestor);
                }
            }
            None => return String::from(ancestor),
        }
    }

    return String::from(ancestor_names.last().expect("no elements found"));
}

fn status() {
    let state = read_state().expect("Error reading state");

    if state.branches.is_empty() {
        println!("No branches yet!");
        return;
    }

    let mut branches_vec: Vec<(&String, &Branch)> = state.branches.iter().collect();
    branches_vec.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    let mut branches_table: Vec<StatusTable> = Vec::new();

    for (key, value) in branches_vec {
        let is_merged = is_merged(&key, value.parent_name.as_str(), value.fork_point.as_str());
        let merged_label = is_merged.label();
        let parent_name = value.parent_name.clone();
        let target = get_target(&key, &state);

        branches_table.push(StatusTable {
            branch: key.to_string(),
            parent: parent_name,
            is_merged: String::from(merged_label),
            target,
        });
    }

    let table = Table::new(branches_table);
    println!("{}", table);
}

fn run_git<I, S>(commands: I) -> (String, Option<i32>)
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

enum MergedStatus {
    Merged,
    NotMerged,
    NoCommits,
    DunnoBro,
}

impl MergedStatus {
    fn label(&self) -> &str {
        match self {
            MergedStatus::Merged => "merged",
            MergedStatus::NotMerged => "not merged",
            MergedStatus::NoCommits => "not merged, no commits",
            MergedStatus::DunnoBro => "gone/unsure",
        }
    }
}

fn is_merged(branch: &str, parent: &str, fork_point: &str) -> MergedStatus {
    let (tip, _) = run_git(["rev-parse", branch]);
    if tip.trim() == fork_point {
        return MergedStatus::NoCommits;
    }
    let (_, status) = run_git(["merge-base", "--is-ancestor", branch, parent]);

    match status {
        Some(0) => MergedStatus::Merged,
        Some(1) => MergedStatus::NotMerged,
        Some(_) => MergedStatus::DunnoBro,
        None => MergedStatus::DunnoBro,
    }
}

fn hook_post_checkout(_prev: &str, _new: &str, is_branch_checkout: &i32) {
    if *is_branch_checkout == 0 {
        return;
    }

    let (current_branch_name_string, _) = run_git(["rev-parse", "--abbrev-ref", "HEAD"]);

    let mut state = read_state().expect("Error reading state");
    if state
        .branches
        .contains_key(current_branch_name_string.trim())
    {
        return;
    }

    let (reflog_time_output, _) = run_git([
        "log",
        "-g",
        "--format=%gd",
        "--date=unix",
        current_branch_name_string.trim(),
    ]);
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

        if last_reflog_digits <= state.installed_at {
            return;
        }
    } else {
        panic!("Could not parse last reflog time")
    }

    let (previous_branch_name_string, _) = run_git(["rev-parse", "--abbrev-ref", "@{-1}"]);

    if previous_branch_name_string.trim().is_empty() {
        return;
    }

    let now = get_epoch_secs();

    let (fork_point, _) = run_git(["rev-parse", "HEAD"]);

    let current_branch = Branch {
        created_at: now,
        fork_point: String::from(fork_point.trim()),
        parent_name: String::from(previous_branch_name_string.trim()),
    };

    state.branches.insert(
        String::from(current_branch_name_string.trim()),
        current_branch,
    );

    write_state(&state);
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
    }
}
