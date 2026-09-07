use clap::{Args, Parser, Subcommand};

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
    HookPostMerge,
}

#[derive(Args)]
struct HookPostCheckoutArgs {
    ref_prev_head: String,
    ref_new_head: String,
    /// 0 for file checkout, 1 for branch checkout
    is_branch_checkout: i32,
}

fn init() {
    println!("You invoked the init command!");
}

fn status() {
    println!("You invoked the status command!");
}

fn hook_post_checkout(prev: &str, new: &str, flag: &i32) {
    println!("hook post checkout {} {} {}", prev, new, flag);
}

fn hook_post_merge() {
    println!("You invoked the hook_post_merge command!");
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
        Commands::HookPostMerge => {
            hook_post_merge();
        }
    }
}
