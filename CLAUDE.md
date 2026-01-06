# CLAUDE.md
This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
**up2date** is a Rust CLI utility that automates system updates across Linux distributions (Ubuntu, Pop!_OS, Arch, EndeavourOS) and macOS. It orchestrates updates for system packages, Rust toolchain, Neovim plugins, and cargo-installed binaries.

## Commands
### Build
```bash
cargo build              # Debug build
cargo build --release    # Release build
```

### Test
```bash
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test
```

### Lint
```bash
cargo clippy             # Run clippy linter
```

## Code Style
- **Always use braces** for control flow (`if`, `else`, `for`, `while`, etc.), even for single-line bodies
- **Never use `?`** for error propagation; handle errors explicitly
- **Always prefer `match`** over `if let` or other conditional patterns
- **Never use `unwrap()`**; handle `Option` and `Result` explicitly with `match`

## Completion Criteria
- A request is **not complete** until it has a passing test without errors or warnings
- Complex functions must be thoroughly documented

## Architecture
Single-file architecture (`src/main.rs`) with these key components:
- **`App` struct**: Represents a command with arguments
- **`run_status()`**: Execute command, display output in real-time
- **`run_output()`**: Execute command, capture output for processing
- **`run_apps()`**: Run sequence of Apps with visual separators
- **`run_with_response()`**: Run first App, parse output, conditionally run second App with parsed results appended (used for orphan package detection)
- **`run_with_cargo()`**: Parse `cargo install --list`, update each installed binary

### Update Flow
1. **OS-specific package updates** (apt/pacman+yay/brew)
2. **Orphan package cleanup** (Arch only)
3. **rustup update** (all platforms)
4. **Neovim Lazy.nvim sync** (all platforms)
5. **Cargo binary updates** (all platforms)

### Excluded Cargo Apps
`parse_cargo_apps()` skips updating `tm` and `project` - these are local development tools that should not be reinstalled from crates.io.
