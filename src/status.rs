use crate::lineage::MergedStatus;
use crate::palette::{self, Painter};
use crate::state::State;
use std::collections::{HashMap, HashSet};

const TEE: &str = "\u{251c}\u{2500}\u{2500} "; // "├── "
const ELBOW: &str = "\u{2514}\u{2500}\u{2500} "; // "└── "
const PIPE: &str = "\u{2502}   "; // "│   "
const GAP: &str = "    ";

struct Ctx<'a> {
    state: &'a State,
    painter: Painter,
    current: Option<String>,
    lines: Vec<String>,
}

pub fn status() {
    let state = crate::state::read_state().expect("Error reading state");

    if state.branches.is_empty() {
        println!("No branches yet!");
        return;
    }

    let mut ctx = Ctx {
        state: &state,
        painter: Painter::new(),
        current: crate::git::current_branch(),
        lines: Vec::new(),
    };

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
        let line = ctx.root_line(root);
        ctx.lines.push(line);
        ctx.walk(root, String::new(), &children, &mut visited);
    }

    let mut orphans: Vec<&String> = state
        .branches
        .keys()
        .filter(|name| !visited.contains(*name))
        .collect();
    orphans.sort();
    for orphan in orphans {
        if !visited.insert(orphan.clone()) {
            continue;
        }
        let line = ctx.branch_line(orphan);
        ctx.lines.push(line);
        ctx.walk(orphan, String::new(), &children, &mut visited);
    }

    ctx.flush();
}

impl<'a> Ctx<'a> {
    fn walk(
        &mut self,
        parent: &str,
        prefix: String,
        children: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
    ) {
        let Some(kids) = children.get(parent) else {
            return;
        };

        for (i, kid) in kids.iter().enumerate() {
            let is_last = i == kids.len() - 1;
            let connector = self
                .painter
                .fg(if is_last { ELBOW } else { TEE }, palette::FAINT);

            if !visited.insert(kid.clone()) {
                let cycle = self.painter.fg("(cycle)", palette::ORANGE);
                self.lines.push(format!("{prefix}{connector}{kid} {cycle}"));
                continue;
            }

            let body = self.branch_line(kid);
            self.lines.push(format!("{prefix}{connector}{body}"));

            let child_prefix = format!(
                "{prefix}{}",
                self.painter
                    .fg(if is_last { GAP } else { PIPE }, palette::FAINT)
            );
            self.walk(kid, child_prefix, children, visited);
        }
    }

    fn root_line(&self, name: &str) -> String {
        let is_current = self.current.as_deref() == Some(name);
        let name_color = if is_current {
            palette::TEAL
        } else {
            palette::TEXT
        };
        let dot = self.painter.fg(
            "\u{25c9}",
            if is_current {
                palette::TEAL
            } else {
                palette::MUTED
            },
        );
        let painted = self.painter.bold(&self.painter.fg(name, name_color));
        let suffix = self.painter.fg(" (trunk)", palette::FAINT);
        format!("{dot} {painted}{suffix}")
    }

    fn branch_line(&self, name: &str) -> String {
        let is_current = self.current.as_deref() == Some(name);
        let branch = self.state.branches.get(name);

        let status =
            branch.map(|b| crate::lineage::is_merged(name, b.parent_name(), b.fork_point()));

        let dot_color = if is_current {
            palette::TEAL
        } else {
            match &status {
                Some(MergedStatus::Merged) => palette::FAINT,
                Some(MergedStatus::NotMerged) => palette::AMBER,
                Some(MergedStatus::NoCommits) => palette::MUTED,
                Some(MergedStatus::DunnoBro) | None => palette::ORANGE,
            }
        };

        let name_color = if is_current {
            palette::TEAL
        } else {
            match &status {
                Some(MergedStatus::Merged) => palette::FAINT,
                Some(MergedStatus::NoCommits) => palette::MUTED,
                _ => palette::TEXT,
            }
        };
        let mut out = format!(
            "{} {}",
            self.painter.fg("\u{25cf}", dot_color),
            self.painter.fg(name, name_color)
        );

        if let Some(b) = branch {
            out.push_str("  ");
            out.push_str(&self.badge(b.parent_name()));
        }

        if let Some(st) = &status {
            out.push_str("  ");
            out.push_str(&self.status_cell(st));
        }

        out
    }

    fn badge(&self, parent: &str) -> String {
        let label = format!("\u{2190} {parent}");
        self.painter.badge(&label, palette::MUTED)
    }

    fn status_cell(&self, st: &MergedStatus) -> String {
        let (glyph, word, color) = match st {
            MergedStatus::Merged => ("\u{2713}", "merged", palette::FAINT),
            MergedStatus::NotMerged => ("\u{25c6}", "not merged", palette::AMBER),
            MergedStatus::NoCommits => ("\u{2205}", "no commits", palette::MUTED),
            MergedStatus::DunnoBro => ("\u{26a0}\u{fe0e}", "gone / unsure", palette::ORANGE),
        };
        self.painter.fg(&format!("{glyph} {word}"), color)
    }

    fn flush(&self) {
        let title = "Branch Tree";

        println!("{}", self.painter.bold(title));
        println!(
            "{}",
            self.painter
                .fg(&"\u{2500}".repeat(title.chars().count()), palette::FAINT)
        );
        for line in &self.lines {
            println!("{line}");
        }
    }
}
