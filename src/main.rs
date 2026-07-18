use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an mdbook skeleton from a folder of markdown files
    Generate {
        /// Input folder containing markdown files
        input: String,
        /// Output book directory (default: ./book)
        #[arg(short, long)]
        out: Option<String>,
    },
    /// Run `mdbook build` on the generated book
    Build {
        /// Book directory (default: ./book)
        #[arg(short, long)]
        book: Option<String>,
    },
    /// Run `mdbook serve` on the book
    Serve {
        /// Book directory (default: ./book)
        #[arg(short, long)]
        book: Option<String>,
    },
    /// Check for PlantUML jar and Graphviz (dot) and show resolutions
    Check {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate { input, out } => {
            let input = Path::new(input);
            let out = Path::new(out.as_deref().unwrap_or("book"));
            generate_book(input, out)?;
            println!("Generated book at: {}", out.display());
        }
        Commands::Build { book } => {
            // Run quick dependency check and warn if missing
            let problems = check_deps();
            if !problems.is_empty() {
                eprintln!("Dependency check found issues:");
                for p in &problems {
                    eprintln!(" - {}", p);
                }
                eprintln!("Proceeding with mdbook build; rendering PlantUML may fail. Run `mdbook-cv check` for details and resolutions.");
            }

            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_build(book)?;
        }
        Commands::Serve { book } => {
            let problems = check_deps();
            if !problems.is_empty() {
                eprintln!("Dependency check found issues (serving may not render diagrams):");
                for p in &problems {
                    eprintln!(" - {}", p);
                }
                eprintln!("Run `mdbook-cv check` for resolutions.");
            }

            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_serve(book)?;
        }
        Commands::Check {} => {
            let problems = check_deps();
            if problems.is_empty() {
                println!("OK: Graphviz and PlantUML jar appear to be available.");
            } else {
                println!("Found issues:");
                for p in &problems {
                    println!("- {}", p);
                }
                println!("\nResolutions:\n- Install Graphviz (dot). On Debian/Ubuntu: sudo apt-get install graphviz; macOS: brew install graphviz; Windows: use choco or download installer from https://graphviz.org/download/\n- Install Java (JRE/JDK). On Debian/Ubuntu: sudo apt-get install default-jre; macOS: brew install openjdk\n- Download plantuml.jar from https://plantuml.com/download and place it in /opt/plantuml/plantuml.jar or set PLANTUML_JAR environment variable pointing to it. Example: export PLANTUML_JAR=/opt/plantuml/plantuml.jar\n- Alternatively install a system 'plantuml' wrapper command if available.\n");
            }
        }
    }
    Ok(())
}

fn generate_book(input: &Path, out: &Path) -> Result<()> {
    // create book/src
    let src = out.join("src");
    fs::create_dir_all(&src).with_context(|| format!("creating {}", src.display()))?;

    // copy markdown files preserving relative paths
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let p = entry.into_path();
            if let Some(ext) = p.extension() {
                if ext == "md" || ext == "markdown" {
                    // compute relative path from input
                    let rel = p.strip_prefix(input).unwrap();
                    let dest = src.join(rel);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&p, &dest)?;
                    entries.push(rel.to_path_buf());
                }
            }
        }
    }

    // sort entries for deterministic SUMMARY
    entries.sort();

    // write SUMMARY.md
    let mut summary = String::from("# Summary\n\n");
    for e in &entries {
        let title = e.file_stem().and_then(|s| s.to_str()).unwrap_or("Chapter");
        let path = e.to_string_lossy().replace("\\","/");
        summary.push_str(&format!("* [{}]({})\n", title, path));
    }
    fs::write(src.join("SUMMARY.md"), summary)?;

    // write a minimal book.toml enabling mermaid and plantuml plugins (user can customize)
    let book_toml = r#"[book]
title = "Generated Book"
authors = ["mdbook-cv"]

[preprocessor.plantuml]
command = "mdbook-plantuml"

[preprocessor.mermaid]
command = "mdbook-mermaid"
"#;
    fs::write(out.join("book.toml"), book_toml)?;

    Ok(())
}

fn run_mdbook_build(book: &Path) -> Result<()> {
    let status = Command::new("mdbook")
        .arg("build")
        .arg(book)
        .status()
        .context("running mdbook build")?;
    if !status.success() {
        anyhow::bail!("mdbook build failed");
    }
    println!("mdbook build succeeded");
    Ok(())
}

fn run_mdbook_serve(book: &Path) -> Result<()> {
    let mut child = Command::new("mdbook")
        .arg("serve")
        .arg(book)
        .spawn()
        .context("starting mdbook serve")?;
    println!("mdbook serve running — Ctrl+C to stop");
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("mdbook serve exited with error");
    }
    Ok(())
}

fn check_deps() -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    // Check Graphviz (dot)
    match Command::new("dot").arg("-V").output() {
        Ok(_) => { /* found */ }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                problems.push("Graphviz 'dot' not found in PATH.".into());
            } else {
                problems.push(format!("Graphviz 'dot' check failed: {}", e));
            }
        }
    }

    // Check plantuml.jar or plantuml command
    let mut found_plantuml = false;
    if let Ok(p) = env::var("PLANTUML_JAR") {
        if Path::new(&p).exists() {
            found_plantuml = true;
        } else {
            problems.push(format!("PLANTUML_JAR is set to '{}' but file does not exist.", p));
        }
    }

    if !found_plantuml {
        let candidates = vec![
            PathBuf::from("./plantuml.jar"),
            PathBuf::from("/opt/plantuml/plantuml.jar"),
            PathBuf::from(r"C:\\plantuml\\plantuml.jar"),
        ];
        for c in candidates {
            if c.exists() {
                found_plantuml = true;
                break;
            }
        }
    }

    if !found_plantuml {
        // Check 'plantuml' executable as fallback
        match Command::new("plantuml").arg("-version").output() {
            Ok(o) => {
                if o.status.success() {
                    found_plantuml = true;
                }
            }
            Err(_) => {}
        }
    }

    if !found_plantuml {
        problems.push("PlantUML jar not found (no PLANTUML_JAR, ./plantuml.jar, /opt/plantuml/plantuml.jar, or system 'plantuml' command).".into());
    }

    problems
}
