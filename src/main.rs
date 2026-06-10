mod analyzer;
mod generator;
mod parser;
mod types;
mod shellenv;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use types::ShellType;

#[derive(Parser)]
#[command(name = "joshconfig")]
#[command(about = "Analyze shell config files and manage PATH/MANPATH/INFOPATH")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze shell config files and generate .paths.d/.manpaths.d/.infopaths.d
    Analyze {
        /// Shell to analyze (bash, zsh, or both)
        #[arg(value_enum, default_value = "both")]
        shell: ShellArg,

        /// Dry run - show what would be generated without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// List current entries in .paths.d, .manpaths.d, and .infopaths.d
    List,

    /// Clean up old entries that are no longer in config files
    Clean,

    /// Emit shell commands to set PATH, MANPATH, and INFOPATH from .d directories
    Shellenv,

    /// Check and fix loader line position in shell config files
    Doctor {
        /// Fix any issues found (without this flag, only report problems)
        #[arg(long)]
        fix: bool,

        /// Target a specific shell config file
        #[arg(long)]
        shell: Option<ShellArg>,
    },
}
#[derive(clap::ValueEnum, Clone)]
enum ShellArg {
    Bash,
    Zsh,
    Both,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { shell, dry_run } => cmd_analyze(shell, dry_run),
        Commands::List => cmd_list(),
        Commands::Clean => cmd_clean(),
        Commands::Shellenv => {
            let code = shellenv::emit_shell_env();
            std::process::exit(code);
        }
        Commands::Doctor { fix, shell } => cmd_doctor(fix, shell),
    }
}

fn cmd_analyze(shell: ShellArg, dry_run: bool) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    println!("Analyzing shell configuration files...");

    let mut all_entries = Vec::new();

    // Analyze based on selected shell(s)
    match shell {
        ShellArg::Bash => {
            println!("Analyzing bash configuration...");
            let entries = analyzer::analyze_shell(ShellType::Bash)?;
            all_entries.extend(entries);
        }
        ShellArg::Zsh => {
            println!("Analyzing zsh configuration...");
            let entries = analyzer::analyze_shell(ShellType::Zsh)?;
            all_entries.extend(entries);
        }
        ShellArg::Both => {
            println!("Analyzing bash configuration...");
            let bash_entries = analyzer::analyze_shell(ShellType::Bash)?;
            all_entries.extend(bash_entries);

            println!("Analyzing zsh configuration...");
            let zsh_entries = analyzer::analyze_shell(ShellType::Zsh)?;
            all_entries.extend(zsh_entries);
        }
    }

    // Also do static parsing for better line number attribution
    let config_files = [
        ShellType::Bash.config_files(),
        ShellType::Zsh.config_files(),
    ]
    .concat();

    for config_file in config_files {
        if !config_file.exists() {
            continue;
        }

        match parser::parse_shell_file(&config_file) {
            Ok(entries) => {
                // Merge static analysis results (prefer static for line numbers)
                for entry in entries {
                    // Check if we already have this path from dynamic analysis
                    let already_have = all_entries.iter().any(|e| {
                        e.path == entry.path
                            && e.variable == entry.variable
                            && e.source_file == entry.source_file
                    });

                    if !already_have {
                        all_entries.push(entry);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", config_file.display(), e);
            }
        }
    }

    // Assign order numbers
    for (i, entry) in all_entries.iter_mut().enumerate() {
        if entry.order == 0 {
            entry.order = i + 1;
        }
    }

    println!("\nFound {} path entries", all_entries.len());

    if dry_run {
        println!("\nDry run - would generate:");
        for entry in &all_entries {
            println!(
                "  {} -> {} # {}",
                entry.filename(),
                entry.normalized_path(),
                entry.comment
            );
        }
    } else {
        // Clean up old entries first
        generator::cleanup_old_entries(&home, &all_entries)?;

        // Generate new files
        generator::generate_path_files(&all_entries, &home)?;

        println!("\nGenerated files in:");
        println!("  {}", home.join(".paths.d").display());
        println!("  {}", home.join(".manpaths.d").display());
        println!("  {}", home.join(".infopaths.d").display());
    }

    Ok(())
}
fn cmd_list() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    println!("PATH entries ({}):", home.join(".paths.d").display());
    list_dir(&home.join(".paths.d"))?;

    println!("\nMANPATH entries ({}):", home.join(".manpaths.d").display());
    list_dir(&home.join(".manpaths.d"))?;

    println!("\nINFOPATH entries ({}):", home.join(".infopaths.d").display());
    list_dir(&home.join(".infopaths.d"))?;

    Ok(())
}

fn list_dir(dir: &PathBuf) -> Result<()> {
    if !dir.exists() {
        println!("  (directory does not exist)");
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            let content = content.trim();
            if !content.is_empty() {
                println!("  {}: {}", entry.file_name().to_string_lossy(), content);
            }
        }
    }

    Ok(())
}

fn cmd_clean() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    println!("Cleaning up old entries...");

    // Re-analyze to get current entries
    let mut all_entries = Vec::new();

    if let Ok(entries) = analyzer::analyze_shell(ShellType::Bash) {
        all_entries.extend(entries);
    }

    if let Ok(entries) = analyzer::analyze_shell(ShellType::Zsh) {
        all_entries.extend(entries);
    }

    generator::cleanup_old_entries(&home, &all_entries)?;

    println!("Cleanup complete");

    Ok(())
}

fn cmd_doctor(fix: bool, shell: Option<ShellArg>) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    // Determine which config files to check
    let shells: &[ShellArg] = match shell {
        Some(s) => &[s],
        None => &[ShellArg::Bash, ShellArg::Zsh],
    };

    for shell_arg in shells {
        let configs: Vec<std::path::PathBuf> = match shell_arg {
            ShellArg::Bash => vec![
                home.join(".bash_profile"),
                home.join(".bashrc"),
            ],
            ShellArg::Zsh => vec![home.join(".zshrc")],
            ShellArg::Both => vec![
                home.join(".bash_profile"),
                home.join(".bashrc"),
                home.join(".zshrc"),
            ],
        };

        for config in &configs {
            if !config.exists() {
                continue;
            }

            let content = match std::fs::read_to_string(config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Cannot read {}: {}", config.display(), e);
                    continue;
                }
            };

            let lines: Vec<&str> = content.lines().collect();

            // Find the last line containing joshconfig-load-paths.sh
            let loader_idx = lines.iter().rposition(|l| l.contains("joshconfig-load-paths.sh"));

            match loader_idx {
                None => {
                    println!("{}: no joshconfig-load-paths.sh loader line found", config.display());
                    println!("  Add the following line to the END of this file:");
                    println!("  . \"$HOME/.local/bin/joshconfig-load-paths.sh\"");
                }
                Some(idx) => {
                    // Check if any non-empty, non-comment lines follow it
                    let has_content_after = lines[idx + 1..]
                        .iter()
                        .any(|l| {
                            let t = l.trim();
                            !t.is_empty() && !t.starts_with('#')
                        });

                    if has_content_after {
                        println!("{}: loader line is not at the end of the file", config.display());
                        println!("  Line {} (0-indexed) contains the loader, but content follows.", idx);

                        if fix {
                            // Move the loader line to the end
                            let mut new_lines: Vec<&str> = lines.clone();
                            let loader_line = new_lines.remove(idx);
                            new_lines.push(loader_line);
                            let new_content = new_lines.join("\n") + "\n";

                            match std::fs::write(config, &new_content) {
                                Ok(_) => println!("  Fixed: moved loader line to end of {}", config.display()),
                                Err(e) => eprintln!("  Error writing {}: {}", config.display(), e),
                            }
                        } else {
                            println!("  Run with --fix to move it to the end.");
                        }
                    } else {
                        println!("{}: loader line is correctly at the end ✓", config.display());
                    }
                }
            }
        }
    }

    Ok(())
}
