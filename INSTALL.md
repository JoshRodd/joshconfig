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

- **Shell**: zsh or bash (bash loader supports GNU bash 3.2.57+)

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
- Build the `joshconfig` binary in release mode
- Install binary to `~/.local/bin/joshconfig`
- Install loader scripts to `~/.local/bin/load-paths.zsh` and `~/.local/bin/load-paths.bash`
- Create `~/.paths.d/` and `~/.manpaths.d/` directories

### 3. Analyze your shell configuration

```bash
joshconfig analyze
```

This scans your shell config files and creates entries in `~/.paths.d/` and `~/.manpaths.d/`.

### 4. Add the loader to your shell config

Add the correct line to the **end** of your shell config:

```bash
# zsh: add to ~/.zshrc
. "$HOME/.local/bin/load-paths.zsh"

# bash: add to ~/.bashrc
. "$HOME/.local/bin/load-paths.bash"
```

Use the zsh line for zsh, the bash line for bash, or both if you use both shells.

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
```

## Uninstallation

To remove joshconfig:

```bash
# Remove installed files
rm -f ~/.local/bin/joshconfig
rm -f ~/.local/bin/load-paths.zsh
rm -f ~/.local/bin/load-paths.bash

# Optionally remove managed entries
rm -rf ~/.paths.d
rm -rf ~/.manpaths.d

# Remove the loader line from ~/.zshrc and/or ~/.bashrc
# Delete: . "$HOME/.local/bin/load-paths.zsh"
# Delete: . "$HOME/.local/bin/load-paths.bash"

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

### Duplicate paths

If you still see duplicates:
1. Check if the loader is being sourced multiple times
2. Verify the loader line is at the end of your config
3. Run `joshconfig analyze` again to regenerate entries

### Permission denied

Ensure the binary is executable:

```bash
chmod +x ~/.local/bin/joshconfig
chmod +x ~/.local/bin/load-paths.zsh
chmod +x ~/.local/bin/load-paths.bash
```

## Updating

To update to the latest version:

```bash
cd /path/to/joshconfig
git pull
source "$HOME/.cargo/env"
cargo build --release
cp target/release/joshconfig ~/.local/bin/
cp scripts/load-paths.zsh ~/.local/bin/
cp scripts/load-paths.bash ~/.local/bin/
joshconfig analyze
```

## Manual Installation

If you prefer to install manually:

```bash
# Build the binary
source "$HOME/.cargo/env"
cargo build --release

# Install files
mkdir -p ~/.local/bin
cp target/release/joshconfig ~/.local/bin/
cp scripts/load-paths.zsh ~/.local/bin/
cp scripts/load-paths.bash ~/.local/bin/

# Create directories
mkdir -p ~/.paths.d
mkdir -p ~/.manpaths.d

# Analyze configs
~/.local/bin/joshconfig analyze

# Add loader to shell config
echo '. "$HOME/.local/bin/load-paths.zsh"' >> ~/.zshrc
echo '. "$HOME/.local/bin/load-paths.bash"' >> ~/.bashrc
```

## System-wide Installation

For system-wide installation (requires root):

```bash
# Build and install to /usr/local
cargo build --release
sudo cp target/release/joshconfig /usr/local/bin/
sudo cp scripts/load-paths.zsh /usr/local/bin/
sudo cp scripts/load-paths.bash /usr/local/bin/

# Users still need to run:
# joshconfig analyze
# And add the matching loader to their ~/.zshrc and/or ~/.bashrc
```

## Building from Source

To build from source without installing:

```bash
# Debug build
cargo build

# Release build
cargo build --release

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
