use clap::{Parser, Subcommand};
use colored::Colorize;
use pk_core::init::{generate_pipeline_kit_structure, InitOptions};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pipeline-kit")]
#[command(version, about = "AI agent pipeline orchestration CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new .pipeline-kit directory with templates
    Init {
        /// Overwrite existing .pipeline-kit directory
        #[arg(short, long)]
        force: bool,

        /// Create minimal template only (1 agent, 1 pipeline)
        #[arg(short, long)]
        minimal: bool,

        /// Target directory (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // When `pipeline` is called without any arguments, launch the TUI
            pk_tui::run_app()
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e))
        }
        Some(Commands::Init {
            force,
            minimal,
            path,
        }) => {
            // Determine target directory
            let target_dir = path
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            // Create init options
            let options = InitOptions {
                target_dir: target_dir.clone(),
                force,
                minimal,
            };

            // Execute initialization
            match generate_pipeline_kit_structure(options).await {
                Ok(_) => {
                    println!("{}", "✓ Created .pipeline-kit/ directory structure".green());
                    println!("{}", "✓ Generated config.toml".green());

                    if minimal {
                        println!("{}", "✓ Created 1 agent template (developer)".green());
                        println!("{}", "✓ Created 1 pipeline template (simple-task)".green());
                    } else {
                        println!(
                            "{}",
                            "✓ Created 2 agent templates (developer, reviewer)".green()
                        );
                        println!(
                            "{}",
                            "✓ Created 2 pipeline templates (simple-task, code-review)".green()
                        );
                    }

                    println!();
                    println!(
                        "{}",
                        "🎉 Pipeline Kit initialized successfully!".bold().green()
                    );
                    println!();
                    println!("Next steps:");
                    println!("  1. Set up your API keys:");
                    println!("     {}", "export ANTHROPIC_API_KEY=your_api_key".cyan());
                    println!();
                    println!("  2. Launch the TUI:");
                    println!("     {}", "pipeline-kit".cyan());
                    println!();
                    println!("  3. Start a pipeline:");
                    println!(
                        "     {}",
                        "Type '/start simple-task' in the command input".cyan()
                    );
                    println!();
                    println!(
                        "For more information, visit: {}",
                        "https://github.com/Vooster-AI/pipeline-kit".blue()
                    );

                    Ok(())
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_no_args() {
        // Test that no arguments defaults to TUI mode
        let cli = Cli::try_parse_from(["pipeline-kit"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parsing_init() {
        let cli = Cli::try_parse_from(["pipeline-kit", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Init { .. })));
    }

    #[test]
    fn test_cli_parsing_init_with_force() {
        let cli = Cli::try_parse_from(["pipeline-kit", "init", "--force"]).unwrap();
        if let Some(Commands::Init {
            force,
            minimal,
            path,
        }) = cli.command
        {
            assert!(force);
            assert!(!minimal);
            assert!(path.is_none());
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parsing_init_with_minimal() {
        let cli = Cli::try_parse_from(["pipeline-kit", "init", "--minimal"]).unwrap();
        if let Some(Commands::Init {
            force,
            minimal,
            path,
        }) = cli.command
        {
            assert!(!force);
            assert!(minimal);
            assert!(path.is_none());
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parsing_init_with_path() {
        let cli = Cli::try_parse_from(["pipeline-kit", "init", "--path", "/tmp/test"]).unwrap();
        if let Some(Commands::Init {
            force,
            minimal,
            path,
        }) = cli.command
        {
            assert!(!force);
            assert!(!minimal);
            assert_eq!(path, Some(PathBuf::from("/tmp/test")));
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_cli_parsing_init_with_all_flags() {
        let cli = Cli::try_parse_from([
            "pipeline-kit",
            "init",
            "--force",
            "--minimal",
            "--path",
            "/tmp/test",
        ])
        .unwrap();
        if let Some(Commands::Init {
            force,
            minimal,
            path,
        }) = cli.command
        {
            assert!(force);
            assert!(minimal);
            assert_eq!(path, Some(PathBuf::from("/tmp/test")));
        } else {
            panic!("Expected Init command");
        }
    }
}
