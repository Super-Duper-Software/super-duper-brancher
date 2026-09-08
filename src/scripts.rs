use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn write_and_set_perms(path: &str, script: &String) {
    fs::write(path, script).expect("could not write file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .expect("Failed to set file permissions");
    println!("Wrote script to {}", path);
}

pub fn init() {
    let post_checkout_script = crate::cli::get_script("hook-post-checkout");

    // TODO: check if exists, if does, need to surface error to user
    // and maybe allow for a --force flag that will overwrite
    write_and_set_perms(".git/hooks/post-checkout", &post_checkout_script);

    let mut state = crate::state::read_state().expect("Could not read state");
    state.installed_at = crate::time::get_epoch_secs();
    crate::state::write_state(&state);
}

pub fn hook_post_checkout(_prev: &str, _new: &str, is_branch_checkout: &i32) {
    if *is_branch_checkout == 0 {
        return;
    }

    let (current_branch_name_string, _) =
        crate::git::run_git(["rev-parse", "--abbrev-ref", "HEAD"]);

    let mut state = crate::state::read_state().expect("Error reading state");
    if state
        .branches
        .contains_key(current_branch_name_string.trim())
    {
        return;
    }

    let (reflog_time_output, _) = crate::git::run_git([
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

    let (previous_branch_name_string, _) =
        crate::git::run_git(["rev-parse", "--abbrev-ref", "@{-1}"]);

    if previous_branch_name_string.trim().is_empty() {
        return;
    }

    let (fork_point, _) = crate::git::run_git(["rev-parse", "HEAD"]);

    let current_branch = crate::state::Branch::new(previous_branch_name_string, fork_point);

    state.branches.insert(
        String::from(current_branch_name_string.trim()),
        current_branch,
    );

    crate::state::write_state(&state);
}
