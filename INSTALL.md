# Installation Guide

## Prerequisites

- **Rust**: Required to build the binary
  - Install from [rustup.rs](https://rustup.rs/)
  - Or use your system package manager:
    ```bash
    # macOS (Homebrew)
    brew install rust
    
    # Debian/Ubuntu
    sudo apt install rustc cargo
    
    # Fedora
    sudo dnf install rust
    ```

- **Shell**: zsh or bash (bash 3.2.57+)

## Installation Steps

### 1. Clone the repository

```bash
git clone git@gitlab.com:joshrodd/joshconfig.git
cd joshconfig
```

### 2. Run the installer

```bash
./scripts/install.sh
```

This will:
- Build the `joshconfig` and `joshconfig-env` binaries in release mode
- Install binaries to `~/.local/bin/joshconfig` and `~/.local/bin/joshconfig-env`
- Install loader script to `~/.local/bin/joshconfig-load-paths.sh`
- Create `~/.paths.d/` and `~/.manpaths.d/` directories

### 3. Analyze your shell configuration

```bash
joshconfig analyze
```

This scans your shell config files and creates entries in `~/.paths.d/` and `~/.manpaths.d/`.

### 4. Add the loader to your shell config

Add the following line to the **end** of your shell config:

```bash
# ~/.zshrc or ~/.bashrc
. "$HOME/.local/bin/joshconfig-load-paths.sh"
```

This single line works for both zsh and bash.

**Important**: The loader must be at the end of your config file to ensure it processes paths after all other modifications.

### 5. Restart your shell

Open a new terminal or reload your config:

```bash
source ~/.zshrc   # zsh
source ~/.bashrc  # bash
```

## Verification

Check that the loader is working:

```bash
# Should show no duplicates
echo $PATH | tr ':' '\n' | sort | uniq -c | sort -rn

# List managed entries
joshconfig list

# Check loader position
joshconfig doctor
```

## Uninstallation

To remove joshconfig:

```bash
# Remove installed files
rm -f ~/.local/bin/joshconfig
rm -f ~/.local/bin/joshconfig-env
rm -f ~/.local/bin/joshconfig-load-paths.sh

# Optionally remove managed entries
rm -rf ~/.paths.d
rm -rf ~/.manpaths.d

# Remove the loader line from ~/.zshrc and/or ~/.bashrc
# Delete: . "$HOME/.local/bin/joshconfig-load-paths.sh"

# Optionally remove source directory
rm -rf /path/to/joshconfig
```

## Troubleshooting

### "command not found: joshconfig"

Ensure `~/.local/bin` is in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add this to your shell config before the loader line.

### Paths not being loaded

Check that:
1. The loader line is at the end of your `~/.zshrc` or `~/.bashrc`
2. Files exist in `~/.paths.d/`:
   ```bash
   ls -la ~/.paths.d/
   ```
3. Run `joshconfig analyze` to regenerate entries
4. Run `joshconfig doctor` to check for issues

### Duplicate paths

If you still see duplicates:
1. Check if the loader is being sourced multiple times
2. Verify the loader line is at the end of your config
3. Run `joshconfig analyze` again to regenerate entries

### Permission denied

Ensure the binaries are executable:

```bash
chmod +x ~/.local/bin/joshconfig
chmod +x ~/.local/bin/joshconfig-env
chmod +x ~/.local/bin/joshconfig-load-paths.sh
```

## Updating

To update to the latest version:

```bash
cd /path/to/joshconfig
git pull
source "$HOME/.cargo/env"
cargo build --release --bins
cp target/release/joshconfig ~/.local/bin/
cp target/release/joshconfig-env ~/.local/bin/
cp scripts/joshconfig-load-paths.sh ~/.local/bin/
joshconfig analyze
```

## Manual Installation

If you prefer to install manually:

```bash
# Build the binaries
source "$HOME/.cargo/env"
cargo build --release --bins

# Install files
mkdir -p ~/.local/bin
cp target/release/joshconfig ~/.local/bin/
cp target/release/joshconfig-env ~/.local/bin/
cp scripts/joshconfig-load-paths.sh ~/.local/bin/

# Create directories
mkdir -p ~/.paths.d
mkdir -p ~/.manpaths.d

# Analyze configs
~/.local/bin/joshconfig analyze

# Add loader to shell config
echo '. "$HOME/.local/bin/joshconfig-load-paths.sh"' >> ~/.zshrc
```

## System-wide Installation

For system-wide installation (requires root):

```bash
# Build and install to /usr/local
cargo build --release --bins
sudo cp target/release/joshconfig /usr/local/bin/
sudo cp target/release/joshconfig-env /usr/local/bin/
sudo cp scripts/joshconfig-load-paths.sh /usr/local/bin/

# Users still need to run:
# joshconfig analyze
# And add the loader to their ~/.zshrc and/or ~/.bashrc
```

## Building from Source

To build from source without installing:

```bash
# Debug build
cargo build --bins

# Release build
cargo build --release --bins

# Run tests
cargo test --all

# Run shell tests
./tests/test-loader.sh
```

## Next Steps

- Read the [README.md](README.md) for usage instructions
- Check [TODO.md](TODO.md) for planned features
- View the manpage: `man joshconfig` (after installation)
  - If `man joshconfig` doesn't work, you may need to add `~/.local/share/man` to your `MANPATH`:
    ```bash
    export MANPATH="$HOME/.local/share/man:$MANPATH"
    ```
    Add this to your shell config before the loader line.
