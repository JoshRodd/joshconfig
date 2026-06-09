#!/bin/sh
# install.sh - Install joshconfig PATH/MANPATH manager
# This script installs the binary and shell loader, and sets up directories

set -e

INSTALL_DIR="$HOME/.local/bin"
PATHS_DIR="$HOME/.paths.d"
MANPATHS_DIR="$HOME/.manpaths.d"
LOADER_SCRIPT="$INSTALL_DIR/load-paths.zsh"
LOADER_LINE='. "$HOME/.local/bin/load-paths.zsh"'

# Version
VERSION="0.1.0"

# Argument parsing
case "$1" in
    --version|-v)
        echo "joshconfig install.sh version $VERSION"
        exit 0
        ;;
    --help|-h)
        cat <<EOF
joshconfig install.sh - Install joshconfig PATH/MANPATH manager

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --version    Print version information

This script will:
    - Build the joshconfig binary from source
    - Install joshconfig to ~/.local/bin/
    - Install load-paths.zsh to ~/.local/bin/
    - Create ~/.paths.d/ and ~/.manpaths.d/ directories
    - Install the manpage to ~/.local/share/man/man1/
    - Add loader line to shell config files (if not present)

For more information, see INSTALL.md
EOF
        exit 0
        ;;
esac


echo "Installing joshconfig PATH/MANPATH manager..."
echo

# Create installation directory
echo "Creating $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"

# Create .paths.d and .manpaths.d directories
echo "Creating $PATHS_DIR and $MANPATHS_DIR..."
mkdir -p "$PATHS_DIR"
mkdir -p "$MANPATHS_DIR"

# Build the Rust binary
echo "Building joshconfig binary..."
if command -v cargo >/dev/null 2>&1; then
    cargo build --release
    cp target/release/joshconfig "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/joshconfig"
    echo "Installed joshconfig to $INSTALL_DIR/joshconfig"
else
    echo "Error: cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi


# Copy the loader script
echo "Installing load-paths.zsh..."
cp scripts/load-paths.zsh "$LOADER_SCRIPT"
chmod +x "$LOADER_SCRIPT"

# Install manpage
echo "Installing manpage..."
MAN_DIR="$HOME/.local/share/man/man1"
mkdir -p "$MAN_DIR"
cp man/joshconfig.1 "$MAN_DIR/"
echo "Installed manpage to $MAN_DIR/joshconfig.1"
echo "Run 'man joshconfig' to view it."

echo
echo "Installation complete!"
echo
echo "Next steps:"
echo "1. Run 'joshconfig analyze' to analyze your shell configs and generate path files"
echo
echo "2. Add the following line to the END of your ~/.zshrc:"
echo "   $LOADER_LINE"
echo
echo "   Note: Bash support is not yet implemented. See TODO.md for details."
echo
echo "The loader must be at the END of your config files to ensure"
echo "it processes paths after all other modifications."
echo

# Function to check if loader line is at the end of a file
check_config_file() {
    _file="$1"
    if [ ! -f "$_file" ]; then
        return 0
    fi

    # Check if loader line exists
    if ! grep -qF "$LOADER_LINE" "$_file"; then
        echo "Warning: $LOADER_LINE not found in $_file"
        echo "Please add it manually to the end of the file."
        return 0
    fi

    # Check if there's anything after the loader line
    # Get line number of loader line
    _loader_line=$(grep -nF "$LOADER_LINE" "$_file" | tail -1 | cut -d: -f1)

    if [ -z "$_loader_line" ]; then
        return 0
    fi

    # Get total lines in file
    _total_lines=$(wc -l < "$_file" | tr -d ' ')

    # Check if there are non-empty lines after the loader
    _lines_after=$((_total_lines - _loader_line))

    if [ "$_lines_after" -gt 0 ]; then
        # Check if any of those lines are non-empty and non-comment
        _tail_content=$(tail -n "$_lines_after" "$_file" | grep -v '^[[:space:]]*$' | grep -v '^[[:space:]]*#' || true)

        if [ -n "$_tail_content" ]; then
            echo
            echo "WARNING: Found content after loader line in $_file"
            echo "The loader line should be at the END of the file."
            echo "Please move any PATH/MANPATH modifications before the loader line."
        fi
    fi
}

# Check existing config files
echo "Checking existing shell config files..."
# check_config_file "$HOME/.bashrc"  # TODO: Enable when bash loader is implemented
check_config_file "$HOME/.zshrc"

echo
echo "Installation complete!"
