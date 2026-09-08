pub enum MergedStatus {
    Merged,
    NotMerged,
    NoCommits,
    DunnoBro,
}

impl MergedStatus {
    pub fn label(&self) -> &str {
        match self {
            MergedStatus::Merged => "merged",
            MergedStatus::NotMerged => "not merged",
            MergedStatus::NoCommits => "not merged, no commits",
            MergedStatus::DunnoBro => "gone/unsure",
        }
    }
}

pub fn get_target(branch_name: &str, state: &crate::state::State) -> String {
    let branch = state
        .branches
        .get(branch_name)
        .expect("could not get branch in state");
    let mut ancestor_names: Vec<String> = Vec::new();
    let mut current_parent_name = branch.parent_name();
    let mut current_parent = state.branches.get(branch.parent_name());

    while current_parent.is_some() {
        match current_parent {
            Some(b) => {
                ancestor_names.push(String::from(current_parent_name));
                current_parent_name = b.parent_name();
                current_parent = state.branches.get(b.parent_name());
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
                let (tip, _) = crate::git::run_git(["rev-parse", ancestor]);
                if tip.trim() == ancestor_branch.fork_point() {
                    return String::from(ancestor);
                }
                for later in ancestor_names[index + 1..].iter() {
                    if crate::git::is_ancestor(ancestor, later) {
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

    String::from(ancestor_names.last().expect("no elements found"))
}

pub fn is_merged(branch: &str, parent: &str, fork_point: &str) -> MergedStatus {
    let (tip, _) = crate::git::run_git(["rev-parse", branch]);
    if tip.trim() == fork_point {
        return MergedStatus::NoCommits;
    }
    let (_, status) = crate::git::run_git(["merge-base", "--is-ancestor", branch, parent]);

    match status {
        Some(0) => MergedStatus::Merged,
        Some(1) => MergedStatus::NotMerged,
        Some(_) => MergedStatus::DunnoBro,
        None => MergedStatus::DunnoBro,
    }
}
