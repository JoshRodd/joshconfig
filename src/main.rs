mod analyzer;
mod generator;
mod parser;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use types::ShellType;

#[derive(Parser)]
#[command(name = "joshconfig")]
#[command(about = "Analyze shell config files and manage PATH/MANPATH")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze shell config files and generate .paths.d/.manpaths.d
    Analyze {
        /// Shell to analyze (bash, zsh, or both)
        #[arg(value_enum, default_value = "both")]
        shell: ShellArg,

        /// Dry run - show what would be generated without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// List current entries in .paths.d and .manpaths.d
    List,

    /// Clean up old entries that are no longer in config files
    Clean,
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
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    println!("PATH entries ({}):", home.join(".paths.d").display());
    list_dir(&home.join(".paths.d"))?;

    println!("\nMANPATH entries ({}):", home.join(".manpaths.d").display());
    list_dir(&home.join(".manpaths.d"))?;

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
