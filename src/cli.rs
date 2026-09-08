use clap::{Args, Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(name = "Super Duper Brancher")]
#[command(version = "1.0")]
#[command(about = "Small cli to keep track of branch parents & merge targets", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Status,
    HookPostCheckout(HookPostCheckoutArgs),
}

#[derive(Args)]
pub struct HookPostCheckoutArgs {
    pub ref_prev_head: String,
    pub ref_new_head: String,
    /// 0 for file checkout, 1 for branch checkout
    pub is_branch_checkout: i32,
}

pub fn create_cli() -> Cli {
    Cli::parse()
}

pub fn get_script(subcommand: &str) -> String {
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
