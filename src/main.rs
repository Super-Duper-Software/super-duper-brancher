use std::fs;
use std::os::unix::fs::PermissionsExt;

mod cli;
mod git;
mod lineage;
mod scripts;
mod state;
mod status;
mod time;

fn write_and_set_perms(path: &str, script: &String) {
    fs::write(path, script).expect("could not write file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .expect("Failed to set file permissions");
    println!("Wrote script to {}", path);
}

fn init() {
    let post_checkout_script = cli::get_script("hook-post-checkout");

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);

    let mut state = state::read_state().expect("Could not read state");
    state.installed_at = time::get_epoch_secs();
    state::write_state(&state);
}

fn main() {
    let cli = cli::create_cli();

    match &cli.command {
        cli::Commands::Init => init(),
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
