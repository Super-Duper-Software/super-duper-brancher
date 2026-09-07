use clap::{Args, Parser, Subcommand};
use std::os::unix::fs::PermissionsExt;
use std::{env, fs};

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

fn init() {
    let post_checkout_script = get_script("hook-post-checkout");
    let post_merge_script = get_script("hook-post-merge");

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);
    write_and_set_perms(".git/hooks/post-merge", &post_merge_script);
}

fn status() {
    println!("You invoked the status command!");
}

fn hook_post_checkout(prev: &str, new: &str, is_branch_checkout: &i32) {
    println!(
        "hook post checkout. prev: {}, new: {}, is_branch_checkout: {}",
        prev, new, is_branch_checkout
    );
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
