use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use context_manager::Config;
use context_manager::Template;

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
    /// Create a new project or work context
    New {
        /// What to create: `project` or `context`; prompted when omitted
        #[command(subcommand)]
        kind: Option<NewKind>,
    },
    /// List the projects inside the work context repo
    List,
    /// Open a work context from a project
    Open {
        /// Project name; prompted when omitted
        project: Option<String>,
    },
    /// Browse projects and contexts interactively
    Tree,
    /// Print the current configuration
    ShowConfig,
}

#[derive(Subcommand)]
enum NewKind {
    /// Create a new project folder
    Project {
        /// Project name; prompted when omitted
        name: Option<String>,
    },
    /// Create a new work context inside a project
    Context {
        /// Work name; prompted when omitted
        name: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init => cmd_init(),
        Command::New { kind } => cmd_new(kind),
        Command::List => cmd_list(),
        Command::Open { project } => cmd_open(project),
        Command::Tree => cmd_tree(),
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

fn cmd_new(kind: Option<NewKind>) -> Result<()> {
    let cfg = load_or_create_config()?;
    match kind {
        Some(NewKind::Project { name }) => cmd_new_project(&cfg, name),
        Some(NewKind::Context { name }) => cmd_new_context(&cfg, name),
        None => {
            use dialoguer::Select;

            let items = ["project", "context"];
            let selection = Select::with_theme(&theme())
                .with_prompt("What do you want to create?")
                .items(items)
                .interact()
                .context("failed to read selection")?;
            match items[selection] {
                "project" => cmd_new_project(&cfg, None),
                _ => cmd_new_context(&cfg, None),
            }
        }
    }
}

fn cmd_new_project(cfg: &Config, name: Option<String>) -> Result<()> {
    let name = match name {
        Some(name) => name,
        None => prompt_text("Project name")?,
    };
    let path = context_manager::project::create_project(cfg, &name)
        .with_context(|| "failed to create project")?;
    println!(
        "{} created project at {}",
        "✓".green().bold(),
        path.display()
    );
    Ok(())
}

fn cmd_new_context(cfg: &Config, name: Option<String>) -> Result<()> {
    let projects = context_manager::project::list_projects(cfg).with_context(|| {
        format!(
            "failed to list projects from {}",
            cfg.work_context_repo.display()
        )
    })?;
    if projects.is_empty() {
        return Err(context_manager::Error::NoProjects(cfg.work_context_repo.clone()).into());
    }
    let project = pick_project(&projects)?;

    let name = match name {
        Some(name) => name,
        None => prompt_text("Work name")?,
    };

    let templates =
        context_manager::template::list_templates(&cfg.template_folder).with_context(|| {
            format!(
                "failed to list templates from {}",
                cfg.template_folder.display()
            )
        })?;
    if templates.is_empty() {
        return Err(context_manager::Error::NoTemplates(cfg.template_folder.clone()).into());
    }

    let template = pick_template(&templates)?;
    let path = context_manager::work_context::new_work_context(cfg, &project, &name, &template)
        .with_context(|| "failed to create work context")?;
    println!(
        "{} created work context at {}",
        "✓".green().bold(),
        path.display()
    );

    let editor = cfg.resolve_editor();
    println!("{} opening with `{}` ...", "➜".cyan().bold(), editor);
    context_manager::open_with(&path, &editor)
        .with_context(|| "failed to open the work context in the editor")?;
    Ok(())
}

fn cmd_list() -> Result<()> {
    let cfg = load_or_create_config()?;
    let projects = context_manager::project::list_projects(&cfg).with_context(|| {
        format!(
            "failed to list projects from {}",
            cfg.work_context_repo.display()
        )
    })?;
    if projects.is_empty() {
        println!("no projects found in {}", cfg.work_context_repo.display());
        println!("run `context-manager new project` to create one");
        return Ok(());
    }
    for project in projects {
        println!("{project}");
    }
    Ok(())
}

fn cmd_open(project: Option<String>) -> Result<()> {
    let cfg = load_or_create_config()?;
    let project = match project {
        Some(project) => project,
        None => {
            let projects = context_manager::project::list_projects(&cfg).with_context(|| {
                format!(
                    "failed to list projects from {}",
                    cfg.work_context_repo.display()
                )
            })?;
            if projects.is_empty() {
                return Err(
                    context_manager::Error::NoProjects(cfg.work_context_repo.clone()).into(),
                );
            }
            pick_project(&projects)?
        }
    };

    let contexts = context_manager::project::list_contexts(&cfg, &project)
        .with_context(|| format!("failed to list contexts in project `{project}`"))?;
    if contexts.is_empty() {
        println!("no work contexts found in project `{project}`");
        return Ok(());
    }
    let context = pick_context(&contexts)?;

    let editor = cfg.resolve_editor();
    println!("{} opening with `{}` ...", "➜".cyan().bold(), editor);
    context_manager::open_with(&context, &editor)
        .with_context(|| "failed to open the work context in the editor")?;
    Ok(())
}

fn cmd_tree() -> Result<()> {
    use console::{Key, Term};
    use std::io::Write;

    let cfg = load_or_create_config()?;
    let root = context_manager::tree::build_tree(&cfg)?;
    let mut term = Term::stderr();

    let mut levels: Vec<(context_manager::tree::TreeNode, usize)> = vec![(root, 0)];

    term.hide_cursor()?;
    let result = (|| -> Result<()> {
        loop {
            term.clear_screen()?;
            render_help(&mut term)?;
            {
                let (node, cursor) = levels.last().expect("tree never empty");
                writeln!(
                    term,
                    "{} {}",
                    "📁".cyan().bold(),
                    breadcrumb(levels.last().expect("tree never empty").0.path.as_path()).bold()
                )?;
                writeln!(term)?;
                if node.children.is_empty() {
                    writeln!(term, "{}", "(no entries)".dimmed())?;
                } else {
                    for (i, child) in node.children.iter().enumerate() {
                        let marker = if i == *cursor { "❯" } else { " " };
                        if child.is_dir() {
                            writeln!(
                                term,
                                "{} {}/",
                                marker.green().bold(),
                                child.name.cyan().bold()
                            )?;
                        } else {
                            writeln!(term, "{} {}", marker.green().bold(), child.name.yellow())?;
                        }
                    }
                }
                term.flush()?;
            }

            match term.read_key()? {
                Key::ArrowUp => {
                    let (_, cursor) = levels.last_mut().expect("tree never empty");
                    if *cursor > 0 {
                        *cursor -= 1;
                    }
                }
                Key::ArrowDown => {
                    let (node, cursor) = levels.last_mut().expect("tree never empty");
                    if *cursor + 1 < node.children.len() {
                        *cursor += 1;
                    }
                }
                Key::ArrowLeft => {
                    if levels.len() > 1 {
                        levels.pop();
                    }
                }
                Key::ArrowRight | Key::Enter => {
                    let (node, cursor) = levels.last().expect("tree never empty");
                    if node.children.is_empty() {
                        continue;
                    }
                    let child = node.children[*cursor].clone();
                    if child.is_dir() {
                        levels.push((child, 0));
                    } else {
                        let editor = cfg.resolve_editor();
                        println!("{} opening with `{}` ...", "➜".cyan().bold(), editor);
                        context_manager::open_with(&child.path, &editor)
                            .with_context(|| "failed to open the work context in the editor")?;
                    }
                }
                Key::Escape => break,
                _ => {}
            }
        }
        Ok(())
    })();
    term.show_cursor()?;
    term.clear_screen()?;
    result
}

fn breadcrumb(path: &std::path::Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if let Ok(stripped) = path.strip_prefix(&home) {
            if stripped.as_os_str().is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

fn render_help(term: &mut console::Term) -> Result<()> {
    use std::io::Write;

    writeln!(
        term,
        "{}",
        "↑/↓ move · →/↵ open · ← back · esc quit".dimmed()
    )?;
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

fn load_or_create_config() -> Result<Config> {
    match Config::load() {
        Ok(cfg) => Ok(cfg),
        Err(_) => {
            let default = Config::default_config()?;
            default.save()?;
            Ok(default)
        }
    }
}

fn theme() -> dialoguer::theme::ColorfulTheme {
    use console::Style;
    use dialoguer::theme::ColorfulTheme;

    ColorfulTheme {
        prompt_style: Style::new().for_stderr().cyan().bold(),
        prompt_prefix: console::style("?".to_string()).for_stderr().cyan().bold(),
        prompt_suffix: console::style("›".to_string()).for_stderr().cyan().bold(),
        active_item_style: Style::new().for_stderr().cyan().bold(),
        active_item_prefix: console::style("❯".to_string()).for_stderr().green().bold(),
        inactive_item_style: Style::new().for_stderr().bright().black(),
        values_style: Style::new().for_stderr().yellow(),
        checked_item_prefix: console::style("✔".to_string()).for_stderr().green(),
        unchecked_item_prefix: console::style("⬚".to_string()).for_stderr().yellow(),
        picked_item_prefix: console::style("❯".to_string()).for_stderr().green().bold(),
        unpicked_item_prefix: console::style(" ".to_string()).for_stderr(),
        ..ColorfulTheme::default()
    }
}

fn prompt_text(prompt: &str) -> Result<String> {
    use dialoguer::Input;

    let name: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .interact_text()
        .with_context(|| format!("failed to read {prompt}"))?;
    Ok(name)
}

fn pick_project(projects: &[String]) -> Result<String> {
    use dialoguer::Select;

    let selection = Select::with_theme(&theme())
        .with_prompt("Choose a project")
        .items(projects)
        .interact()
        .context("failed to read project selection")?;
    Ok(projects[selection].clone())
}

fn pick_context(contexts: &[std::path::PathBuf]) -> Result<std::path::PathBuf> {
    use dialoguer::Select;

    let names: Vec<String> = contexts
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect();
    let selection = Select::with_theme(&theme())
        .with_prompt("Choose a work context")
        .items(&names)
        .interact()
        .context("failed to read context selection")?;
    Ok(contexts[selection].clone())
}

fn pick_template(templates: &[Template]) -> Result<Template> {
    use dialoguer::Select;

    let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
    let selection = Select::with_theme(&theme())
        .with_prompt("Choose a template")
        .items(&names)
        .interact()
        .context("failed to read template selection")?;
    Ok(templates[selection].clone())
}
