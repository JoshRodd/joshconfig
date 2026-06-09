# joshconfig

A tool for managing PATH and MANPATH environment variables through declarative configuration files.

## Overview

joshconfig extracts PATH and MANPATH modifications from shell configuration files (`.bashrc`, `.zshrc`, etc.) and creates individual entry files in `~/.paths.d/` and `~/.manpaths.d/`. A lightweight loader script sources these entries at shell startup, providing deduplication and consistent path management.

## Features

- **Automatic extraction**: Analyzes shell config files using both static parsing and dynamic instrumentation
- **Deduplication**: Removes duplicate path entries while preserving order
- **Shell compatibility**: Supports both zsh and bash (bash 3.2.57+)
- **Transparent format**: Plain text files that are easy to inspect and edit
- **macOS aware**: Skips paths from `/etc/paths.d/` and `/etc/manpaths.d/` (handled by `path_helper`)

## Quick Start

```bash
# Clone and install
git clone git@gitlab.com:joshrodd/joshconfig.git
cd joshconfig
./scripts/install.sh

# Analyze your shell configs
joshconfig analyze

# Add to your ~/.zshrc (for zsh)
echo '. "$HOME/.local/bin/load-paths.zsh"' >> ~/.zshrc

# Or add to your ~/.bashrc (for bash)
echo '. "$HOME/.local/bin/load-paths.bash"' >> ~/.bashrc
```

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

## Usage

### Analyze shell configurations

```bash
joshconfig analyze
```

This scans your shell config files (`.bash_profile`, `.bashrc`, `.zshrc`, etc.) and creates entries in `~/.paths.d/` and `~/.manpaths.d/`.

### List current entries

```bash
joshconfig list
```

Shows all entries in your `.paths.d` and `.manpaths.d` directories.

### Manual entry management

You can manually create, edit, or delete entries:

```bash
# Create a new path entry
echo "/usr/local/go/bin # Go installation" > ~/.paths.d/50-go-bin

# Edit an entry
vim ~/.paths.d/10-homebrew-bin

# Remove an entry
rm ~/.paths.d/20-old-path
```

Entries are sorted numerically by filename prefix (e.g., `10-`, `20-`, `50-`).

## How It Works

### Analysis Phase

joshconfig uses two methods to extract PATH modifications:

1. **Static parsing**: Regex-based extraction of `PATH=` and `MANPATH=` assignments
2. **Dynamic instrumentation**: Spawns shell processes with tracing to capture actual runtime values

### Loader Phase

The loader scripts (`load-paths.zsh` and `load-paths.bash`):
1. Read all files in `~/.paths.d/` (sorted alphabetically)
2. Strip comments and expand `~` and `$HOME`
3. Prepend entries to existing PATH
4. Remove duplicates while preserving order

The zsh and bash loaders are separate ports so each shell can use native, reliable path splitting. The bash loader is compatible with bash 3.2.57+.

## Entry File Format

Each file in `~/.paths.d/` or `~/.manpaths.d/` contains one path entry:

```
/path/to/directory # Optional comment
```

Filename format: `NN-description` where `NN` is a numeric prefix for ordering.

## Limitations

- **No shell interpolation**: Entries cannot contain complex shell expressions
- **Order dependency**: Paths are prepended, so they override existing entries

## License

MIT License. See [LICENSE.md](LICENSE.md) for details.

## Contributing

Contributions welcome! See [TODO.md](TODO.md) for planned features.

## Related Projects

- [direnv](https://direnv.net/) - Environment variable management per directory
- [path_helper](https://ss64.com/osx/path_helper.html) - macOS system path management
