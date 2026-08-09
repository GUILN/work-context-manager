use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use work_context_manager::Config;
use work_context_manager::Template;

#[derive(Parser)]
#[command(name = "context-manager", version, about = "Manage your work contexts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize the config and default template folder
    Init,
    /// Create a new work context
    New {
        /// Work name; prompted when omitted
        name: Option<String>,
    },
    /// Print the current configuration
    ShowConfig,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init => cmd_init(),
        Command::New { name } => cmd_new(name),
        Command::ShowConfig => cmd_show_config(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err:#}", "error:".red().bold());
            ExitCode::FAILURE
        }
    }
}

fn cmd_init() -> Result<()> {
    let cfg = Config::default_config()?;
    cfg.save()?;
    std::fs::create_dir_all(&cfg.template_folder)?;
    println!(
        "{} config at {}",
        "created".green().bold(),
        Config::config_path()?.display()
    );
    Ok(())
}

fn cmd_new(name: Option<String>) -> Result<()> {
    let cfg = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            let default = Config::default_config()?;
            default.save()?;
            default
        }
    };

    let name = match name {
        Some(name) => name,
        None => prompt_name()?,
    };

    let templates = work_context_manager::template::list_templates(&cfg.template_folder)
        .with_context(|| {
            format!(
                "failed to list templates from {}",
                cfg.template_folder.display()
            )
        })?;
    if templates.is_empty() {
        return Err(work_context_manager::Error::NoTemplates(cfg.template_folder.clone()).into());
    }

    let template = pick_template(&templates)?;
    let path = work_context_manager::work_context::new_work_context(&cfg, &name, &template)
        .with_context(|| "failed to create work context")?;
    println!(
        "{} created work context at {}",
        "✓".green().bold(),
        path.display()
    );
    Ok(())
}

fn cmd_show_config() -> Result<()> {
    match Config::load() {
        Ok(cfg) => {
            println!(
                "{} {}",
                "template folder:".bold(),
                cfg.template_folder.display()
            );
            println!(
                "{} {}",
                "work context repo:".bold(),
                cfg.work_context_repo.display()
            );
            Ok(())
        }
        Err(_) => {
            println!("config not found at {}", Config::config_path()?.display());
            println!("run `context-manager init` to create it");
            Ok(())
        }
    }
}

fn prompt_name() -> Result<String> {
    use dialoguer::Input;

    let name: String = Input::new()
        .with_prompt("Work name")
        .interact_text()
        .context("failed to read work name")?;
    Ok(name)
}

fn pick_template(templates: &[Template]) -> Result<Template> {
    use dialoguer::Select;

    let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
    let selection = Select::new()
        .with_prompt("Choose a template")
        .items(&names)
        .interact()
        .context("failed to read template selection")?;
    Ok(templates[selection].clone())
}
