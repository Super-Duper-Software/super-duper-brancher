mod cli;
mod git;
mod lineage;
mod palette;
mod scripts;
mod state;
mod status;
mod time;

fn main() {
    let cli = cli::create_cli();

    match &cli.command {
        cli::Commands::Init(args) => scripts::init(args.force),
        cli::Commands::Status => status::status(),
        cli::Commands::HookPostCheckout(cli::HookPostCheckoutArgs {
            ref_prev_head,
            ref_new_head,
            is_branch_checkout,
        }) => {
            scripts::hook_post_checkout(ref_prev_head, ref_new_head, is_branch_checkout);
        }
    }
}
