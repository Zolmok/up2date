# up2date

A single-command utility to keep Linux and macOS systems up to date. Runs system package updates, Rust toolchain updates, Neovim plugin sync, and cargo-installed binary updates.

## Supported Platforms

| Platform | Package Manager |
|----------|-----------------|
| Ubuntu / Pop!_OS | apt |
| Arch / Omarchy | pacman + yay |
| macOS | Homebrew |

## What it Updates

1. **System packages** - platform-specific package manager updates
2. **Orphan cleanup** - removes unused dependencies (Arch only)
3. **Rust toolchain** - `rustup update` (skipped if rustup not installed)
4. **Neovim plugins** - `Lazy.nvim` sync
5. **Cargo binaries** - updates all cargo-installed packages from crates.io (local path installs are skipped)

## Installation

```bash
cargo install --path .
```

Or to install from the repository:

```bash
git clone https://github.com/zolmok/up2date.git
cd up2date
cargo install --path .
```

## Usage

```bash
up2date
```

That's it. The command runs through all updates sequentially, displaying each command as it executes.

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```
