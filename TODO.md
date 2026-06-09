# TODO

## High Priority

- [ ] Add comprehensive test coverage for shell loader edge cases
  - Paths with spaces
  - Paths with special characters
  - Very long PATH values
  - Empty path entries

## Medium Priority

- [ ] Improve error handling in the analyzer
  - Better error messages when config files can't be read
  - Handle permission errors gracefully
  - Add validation for generated path entries

- [ ] Add support for other path variables
  - CPATH
  - LD_LIBRARY_PATH
  - DYLD_LIBRARY_PATH (macOS)
  - PYTHONPATH

## Low Priority

- [ ] Add a `--watch` mode to the analyzer that monitors config files and regenerates paths automatically
- [ ] Create a TUI (Terminal User Interface) for managing paths interactively
- [ ] Add backup/restore functionality for path configurations
- [ ] Implement path validation (check if paths exist, are directories, etc.)
- [ ] Add shell completion scripts for bash and zsh
- [ ] Add CI/CD pipeline with automated testing
- [ ] Package for Homebrew (macOS) and apt (Debian/Ubuntu)
