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
            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_build(book)?;
        }
        Commands::Serve { book } => {
            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_serve(book)?;
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
