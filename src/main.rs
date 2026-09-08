use clap::{Args, Parser, Subcommand};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tabled::{Table, Tabled};

mod git;
mod lineage;
mod state;
mod time;

fn status() {
    let state = crate::state::read_state().expect("Error reading state");

    if state.branches.is_empty() {
        println!("No branches yet!");
        return;
    }

    let mut branches_vec: Vec<(&String, &crate::state::Branch)> = state.branches.iter().collect();
    branches_vec.sort_by(|a, b| b.1.created_at().cmp(&a.1.created_at()));

    let mut branches_table: Vec<StatusTable> = Vec::new();

    for (key, value) in branches_vec {
        let is_merged = lineage::is_merged(&key, value.parent_name(), value.fork_point());
        let merged_label = is_merged.label();
        let target = lineage::get_target(&key, &state);

        branches_table.push(StatusTable {
            branch: key.to_string(),
            parent: String::from(value.parent_name()),
            is_merged: String::from(merged_label),
            target,
        });
    }

    let table = Table::new(branches_table);
    println!("{}", table);
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

fn init() {
    let post_checkout_script = get_script("hook-post-checkout");

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);

    let mut state = state::read_state().expect("Could not read state");
    state.installed_at = time::get_epoch_secs();
    state::write_state(&state);
}

#[derive(Tabled)]
struct StatusTable {
    branch: String,
    target: String,
    parent: String,
    is_merged: String,
}

fn hook_post_checkout(_prev: &str, _new: &str, is_branch_checkout: &i32) {
    if *is_branch_checkout == 0 {
        return;
    }

    let (current_branch_name_string, _) = git::run_git(["rev-parse", "--abbrev-ref", "HEAD"]);

    let mut state = state::read_state().expect("Error reading state");
    if state
        .branches
        .contains_key(current_branch_name_string.trim())
    {
        return;
    }

    let (reflog_time_output, _) = git::run_git([
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

    let (previous_branch_name_string, _) = git::run_git(["rev-parse", "--abbrev-ref", "@{-1}"]);

    if previous_branch_name_string.trim().is_empty() {
        return;
    }

    let (fork_point, _) = git::run_git(["rev-parse", "HEAD"]);

    let current_branch = state::Branch::new(previous_branch_name_string, fork_point);

    state.branches.insert(
        String::from(current_branch_name_string.trim()),
        current_branch,
    );

    state::write_state(&state);
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
