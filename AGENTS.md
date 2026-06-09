# Development Guidelines for AI Agents

## Shell Script Compatibility

All bash scripts must be compatible with:
- Current versions of bash
- GNU bash version 3.2.57(1)-release (arm64-apple-darwin24) - the stock macOS version

This means avoiding:
- Associative arrays (declare -A)
- Mapfile/readarray
- Process substitution in some contexts
- Advanced parameter expansion features not available in 3.2
- `**` globstar (unless explicitly enabled and tested)

When writing shell scripts, test on both modern bash and bash 3.2.57 to ensure compatibility.

## Documentation Synchronization

When updating documentation, maintain consistency across all sources:

### Installation Instructions
When updating installation instructions, update BOTH:
- `INSTALL.md` - User-facing installation guide
- `man/joshconfig.1` - Manpage installation section

### General Documentation
When updating the README, also update the manpage:
- `README.md` - Project overview and usage
- `man/joshconfig.1` - Corresponding sections in the manpage

### Command Output
When changing `--help` output or command behavior:
- Update the manpage (`man/joshconfig.1`) to reflect the new output
- Update README.md if it contains example output
- Update INSTALL.md if installation steps change

## Version Information

Both the main binary and installation script must respond to standard version queries:
- `joshconfig --version` - Prints version from Cargo.toml
- `scripts/install.sh --version` - Prints installation script version
- `scripts/install.sh --help` - Prints usage information

Keep version numbers synchronized between:
- `Cargo.toml` (main project version)
- `man/joshconfig.1` (manpage header)
- Any other version references in documentation

## Testing Requirements

When modifying shell scripts:
- Test with bash 3.2.57 (available at /bin/bash on macOS)
- Test with modern bash (from Homebrew or other sources)
- Run the test suite: `./tests/test-loader.sh`

When modifying Rust code:
- Run `cargo test` to ensure all tests pass
- Run `cargo build --release` to verify release builds work

## File Locations

Key files to remember:
- Main binary source: `src/main.rs`
- Installation script: `scripts/install.sh`
- Loader script: `scripts/load-paths.zsh`
- Manpage: `man/joshconfig.1`
- Tests: `tests/` directory
- Documentation: `README.md`, `INSTALL.md`, `TODO.md`
