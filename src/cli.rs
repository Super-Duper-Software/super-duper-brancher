use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Super Duper Brancher")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Small cli to keep track of branch parents & merge targets", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Status,
    HookPostCheckout(HookPostCheckoutArgs),
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(short, long)]
    pub force: bool,
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
