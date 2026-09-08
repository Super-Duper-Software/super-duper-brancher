use tabled::{Table, Tabled};

#[derive(Tabled)]
struct StatusTable {
    branch: String,
    target: String,
    parent: String,
    is_merged: String,
}

pub fn status() {
    let state = crate::state::read_state().expect("Error reading state");

    if state.branches.is_empty() {
        println!("No branches yet!");
        return;
    }

    let mut branches_vec: Vec<(&String, &crate::state::Branch)> = state.branches.iter().collect();
    branches_vec.sort_by(|a, b| b.1.created_at().cmp(&a.1.created_at()));

    let mut branches_table: Vec<StatusTable> = Vec::new();

    for (key, value) in branches_vec {
        let is_merged = crate::lineage::is_merged(&key, value.parent_name(), value.fork_point());
        let merged_label = is_merged.label();
        let target = crate::lineage::get_target(&key, &state);

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
