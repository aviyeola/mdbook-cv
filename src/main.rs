use std::env;
use std::env;
use std::fs;
use std::io::Write;
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
    /// Check for PlantUML jar and Graphviz (dot) and optionally download plantuml.jar
    Check {
        /// Attempt to download plantuml.jar to a default location if missing
        #[arg(short, long)]
        download: bool,
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
            let status = check_deps_status();
            if !status.is_all_ok() {
                eprintln!("Dependency check found issues:");
                for msg in status.summaries() {
                    eprintln!(" - {}", msg);
                }
                eprintln!("Proceeding with mdbook build; rendering PlantUML may fail. Run `mdbook-cv check` for details and resolutions.");
            }
            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_build(book)?;
        }
        Commands::Serve { book } => {
            let status = check_deps_status();
            if !status.is_all_ok() {
                eprintln!("Dependency check found issues (serving may not render diagrams):");
                for msg in status.summaries() {
                    eprintln!(" - {}", msg);
                }
                eprintln!("Run `mdbook-cv check` for resolutions.");
            }
            let book = Path::new(book.as_deref().unwrap_or("book"));
            run_mdbook_serve(book)?;
        }
        Commands::Check { download } => {
            let mut status = check_deps_status();
            if status.is_all_ok() {
                println!("OK: Graphviz and PlantUML jar appear to be available.");
                return Ok(());
            }

            println!("Found issues:");
            for msg in status.summaries() {
                println!("- {}", msg);
            }

            println!("\nResolutions and installation hints:");
            for hint in status.install_hints() {
                println!("- {}", hint);
            }

            if *download && !status.plantuml_found {
                let dst = default_plantuml_path();
                println!("plantuml.jar will be downloaded to: {}", dst.display());
                // Prompt the user for confirmation
                use std::io::{stdin, stdout};
                print!("Proceed to download plantuml.jar to {}? [y/N]: ", dst.display());
                stdout().flush()?;
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                let ans = input.trim().to_lowercase();
                if ans == "y" || ans == "yes" {
                    match attempt_download_plantuml(&dst) {
                        Ok(()) => println!("Downloaded plantuml.jar to {}. Set PLANTUML_JAR={} or move it to /opt/plantuml/plantuml.jar if you prefer.", dst.display(), dst.display()),
                        Err(e) => eprintln!("Download failed: {}", e),
                    }
                } else {
                    println!("Download cancelled by user.");
                }
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

struct DepStatus {
    graphviz_found: bool,
    java_found: bool,
    plantuml_found: bool,
    plantuml_path: Option<PathBuf>,
}

impl DepStatus {
    fn is_all_ok(&self) -> bool {
        self.graphviz_found && self.java_found && self.plantuml_found
    }

    fn summaries(&self) -> Vec<String> {
        let mut v = Vec::new();
        if !self.graphviz_found {
            v.push("Graphviz 'dot' not found in PATH.".into());
        }
        if !self.java_found {
            v.push("Java (JRE) not found in PATH.".into());
        }
        if !self.plantuml_found {
            v.push("PlantUML jar not found and no system 'plantuml' command available.".into());
        }
        v
    }

    fn install_hints(&self) -> Vec<String> {
        let os = env::consts::OS;
        let mut v = Vec::new();
        match os {
            "linux" => {
                v.push("Graphviz: sudo apt-get install graphviz (Debian/Ubuntu)".into());
                v.push("Java: sudo apt-get install default-jre".into());
                v.push("PlantUML jar: download https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar and place into $HOME/.plantuml/plantuml.jar or /opt/plantuml/plantuml.jar".into());
            }
            "macos" => {
                v.push("Graphviz: brew install graphviz".into());
                v.push("Java: brew install openjdk".into());
                v.push("PlantUML jar: download from https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar and place into $HOME/.plantuml/plantuml.jar".into());
            }
            "windows" => {
                v.push("Graphviz: choco install graphviz or download installer from https://graphviz.org/download/".into());
                v.push("Java: choco install javaruntime or install from https://adoptium.net/".into());
                v.push("PlantUML jar: download from https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar and place into %USERPROFILE%\\.plantuml\\plantuml.jar".into());
            }
            _ => {
                v.push("Graphviz: install dot from your OS package manager or https://graphviz.org/download/".into());
                v.push("Java: install JRE/JDK from https://adoptium.net/ or your OS packages".into());
                v.push("PlantUML jar: download from https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar".into());
            }
        }
        v
    }
}

fn check_deps_status() -> DepStatus {
    // Check Graphviz (dot)
    let graphviz_found = match Command::new("dot").arg("-V").output() {
        Ok(_) => true,
        Err(e) => e.kind() != std::io::ErrorKind::NotFound,
    };

    // Check Java
    let java_found = match Command::new("java").arg("-version").output() {
        Ok(o) => o.status.success() || !o.stderr.is_empty() || !o.stdout.is_empty(),
        Err(e) => e.kind() != std::io::ErrorKind::NotFound,
    };

    // Check plantuml.jar or plantuml command
    let mut plantuml_found = false;
    let mut plantuml_path: Option<PathBuf> = None;

    if let Ok(p) = env::var("PLANTUML_JAR") {
        let pp = PathBuf::from(&p);
        if pp.exists() {
            plantuml_found = true;
            plantuml_path = Some(pp);
        }
    }

    if !plantuml_found {
        let candidates = vec![
            default_plantuml_path(),
            PathBuf::from("./plantuml.jar"),
        ];
        for c in candidates {
            if c.exists() {
                plantuml_found = true;
                plantuml_path = Some(c);
                break;
            }
        }
    }

    if !plantuml_found {
        match Command::new("plantuml").arg("-version").output() {
            Ok(o) => {
                if o.status.success() {
                    plantuml_found = true;
                }
            }
            Err(_) => {}
        }
    }

    DepStatus {
        graphviz_found,
        java_found,
        plantuml_found,
        plantuml_path,
    }
}

fn default_plantuml_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(up) = env::var("USERPROFILE") {
            return PathBuf::from(format!("{}\\.plantuml\\plantuml.jar", up));
        }
        PathBuf::from(r"C:\\plantuml\\plantuml.jar")
    } else {
        if let Some(home) = dirs::home_dir() {
            return home.join(".plantuml").join("plantuml.jar");
        }
        PathBuf::from("/opt/plantuml/plantuml.jar")
    }
}

fn attempt_download_plantuml(dst: &Path) -> Result<()> {
    let url = "https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar";
    let resp = reqwest::blocking::get(url).context("requesting plantuml.jar")?;
    if !resp.status().is_success() {
        anyhow::bail!("failed to download plantuml.jar: HTTP {}", resp.status());
    }
    let bytes = resp.bytes().context("reading response body")?;

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).context("creating plantuml dir")?;
    }
    let mut file = fs::File::create(dst).context("creating plantuml.jar file")?;
    file.write_all(&bytes).context("writing plantuml.jar")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(dst, perms)?;
    }

    Ok(())
}
