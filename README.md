# sdbr — Super Duper Brancher

A small CLI that keeps track of where your git branches came from and where
they're meant to merge back to.

When you stack branches on top of each other, git itself forgets the
relationship the moment the branch is created. `sdbr` records the parent branch
and fork point at checkout time, then shows you the whole tree — including which
branches have already merged.

## Install

```sh
cargo install --path .
```

## Usage

Run once per repo to install the `post-checkout` git hook:

```sh
sdbr init
```

If you already have a `post-checkout` hook, `sdbr init` prints a line to append
to it, or pass `--force` to overwrite.

From then on, every time you create and check out a new branch, `sdbr` records
its parent and fork point in `.git/sdbr-state.json`. Branches that existed before
`init` are left alone.

Show the branch tree:

```sh
sdbr status
```

```
main
├── feature-a
│   └── feature-a-fixup
└── feature-b
```

Branches are dimmed when they've already merged into their target (or have no
commits of their own), and shown in red when the merge state can't be
determined.

## Commands

| Command | Description |
| --- | --- |
| `sdbr init [--force]` | Install the `post-checkout` hook in the current repo |
| `sdbr status` | Print the branch lineage tree |
| `sdbr hook-post-checkout` | Internal — invoked by the git hook |

## How it works

- State lives in `.git/sdbr-state.json` (per repo, not committed).
- The hook fires on branch checkouts. If the branch is new — not already tracked
  and created after `init` — `sdbr` reads the previous branch (`@{-1}`) as the
  parent and `HEAD` as the fork point.
- `sdbr status` walks each branch's ancestry to pick a merge target, skipping
  ancestors that have themselves already merged.

## Requirements

- `git` on your `PATH`
- Unix-like OS (the hook is written as `/bin/sh` and marked executable)
