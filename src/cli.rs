use clap::{Args, Parser, Subcommand};

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
    Viz,
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
