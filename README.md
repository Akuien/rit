# rit

`rit` is a Git re-implementation from scartch in Rust.

The goal of this project is to learn how Git works internally by rebuilding its core concepts step by step: object storage, hashing, trees, commits, references, branches, the index, checkout, status, and diff.
It follows the git source code mirror implemented originally in C at [Original C source code mirror](https://github.com/Akuien/git-ops)

## Current Features Implemented + more to add

- Initialize a repository with `.rit`
- Store Git-style objects using SHA-1 and zlib compression
- Support blob, tree, and commit objects
- Hash files into the object database
- Read and pretty-print objects
- Write recursive tree snapshots
- Create commits from the staging area
- Maintain `HEAD` and branch references
- View commit history with `log`
- Stage files with `add`
- Show working tree state with `status`
- Switch branches and restore commits with `checkout`
- Protect checkout from overwriting uncommitted changes
- Remove tracked files during checkout when switching branches
- Show unstaged changes with `diff`
- Show staged changes with `diff --cached`

## Commands

```bash
rit init
rit add <file>
rit commit -m "message"
rit status
rit diff
rit diff --cached
rit log
rit branch
rit branch <name>
rit checkout <branch-or-commit>
rit hash-object <file>
rit cat-file <hash>
rit write-tree
```

## Repository Structure

A `rit` repository stores its data in `.rit`:

```text
.rit/
├── HEAD
├── config
├── index
├── objects/
└── refs/
    └── heads/
```

## Core Concepts

`rit` follows the same core model as Git:

```text
working tree
    ↓ rit add
index
    ↓ rit commit
commit
    ↓
tree
    ↓
blobs
```

Objects are stored by content hash:

```text
content -> SHA-1 hash -> compressed object file
```

Branches are simple files under:

```text
.rit/refs/heads/
```

`HEAD` points to the currently active branch.

## Example Usage

```bash
rit init

echo "hello" > hello.txt
rit add hello.txt
rit commit -m "initial commit"

echo "second line" >> hello.txt
rit status
rit diff

rit add hello.txt
rit diff --cached
rit commit -m "update hello"

rit branch feature
rit checkout feature
```

## Development

Build the project:

```bash
cargo build
```

Run a command during development:

```bash
cargo run -- status
```

Run the compiled binary:

```bash
target/debug/rit status
```

## Current Limitations

- The index uses a simple JSON format, not Git’s binary index format.
- Diff is a basic line-by-line implementation, not a full Myers diff.
- Remote operations such as clone, fetch, and push are not implemented yet.
- Merge support is not implemented yet.
- Checkout protection is conservative and blocks all untracked files.
