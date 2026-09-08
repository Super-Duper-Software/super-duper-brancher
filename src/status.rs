use crate::lineage::MergedStatus;
use crate::state::State;
use std::collections::{HashMap, HashSet};
use std::io::IsTerminal;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";

const TEE: &str = "\u{251c}\u{2500}\u{2500} "; // "├── "
const ELBOW: &str = "\u{2514}\u{2500}\u{2500} "; // "└── "
const PIPE: &str = "\u{2502}   "; // "│   "
const GAP: &str = "    ";

pub fn status() {
    let state = crate::state::read_state().expect("Error reading state");

    if state.branches.is_empty() {
        println!("No branches yet!");
        return;
    }

    let color = std::io::stdout().is_terminal();

    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for branch_name in state.branches.keys() {
        let target = crate::lineage::get_target(branch_name, &state);
        children
            .entry(target)
            .or_default()
            .push(branch_name.clone());
    }

    for kids in children.values_mut() {
        kids.sort_by_key(|name| {
            std::cmp::Reverse(
                state
                    .branches
                    .get(name)
                    .map(|b| b.created_at())
                    .unwrap_or(0),
            )
        });
    }

    let mut roots: Vec<&String> = children
        .keys()
        .filter(|target| !state.branches.contains_key(*target))
        .collect();
    roots.sort();

    let mut visited: HashSet<String> = HashSet::new();

    for root in &roots {
        visited.insert((*root).clone());
        println!("{}", paint(root, BOLD, color));
        print_kids(root, String::new(), &children, &state, &mut visited, color);
    }

    let mut orphans: Vec<&String> = state
        .branches
        .keys()
        .filter(|name| !visited.contains(*name))
        .collect();
    orphans.sort();
    for orphan in orphans {
        if visited.contains(orphan) {
            continue;
        }
        visited.insert(orphan.clone());
        println!("{}", paint(orphan, BOLD, color));
        print_kids(
            orphan,
            String::new(),
            &children,
            &state,
            &mut visited,
            color,
        );
    }
}

fn print_kids(
    parent: &str,
    prefix: String,
    children: &HashMap<String, Vec<String>>,
    state: &State,
    visited: &mut HashSet<String>,
    color: bool,
) {
    let Some(kids) = children.get(parent) else {
        return;
    };

    for (i, kid) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = paint(if is_last { ELBOW } else { TEE }, DIM, color);

        if !visited.insert(kid.clone()) {
            println!(
                "{prefix}{connector}{} {}",
                kid,
                paint("(cycle)", RED, color)
            );
            continue;
        }

        println!(
            "{prefix}{connector}{}",
            paint(kid, status_color(kid, state), color)
        );

        let branch_prefix = format!(
            "{prefix}{}",
            paint(if is_last { GAP } else { PIPE }, DIM, color)
        );
        print_kids(kid, branch_prefix, children, state, visited, color);
    }
}

fn status_color(name: &str, state: &State) -> &'static str {
    let Some(b) = state.branches.get(name) else {
        return "";
    };
    match crate::lineage::is_merged(name, b.parent_name(), b.fork_point()) {
        MergedStatus::Merged => DIM,
        MergedStatus::NotMerged => "",
        MergedStatus::NoCommits => DIM,
        MergedStatus::DunnoBro => RED,
    }
}

fn paint(text: &str, code: &str, color: bool) -> String {
    if color && !code.is_empty() {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}
