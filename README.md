mdbook-cv
===========

A small Rust CLI to generate an mdBook from a folder of Markdown files and support PlantUML (bundled jar) and Mermaid via mdbook plugins.

Usage (after building):

  mdbook-cv generate <input-dir> [--out <book-dir>]
  mdbook-cv build <book-dir>
  mdbook-cv serve <book-dir>
  mdbook-cv check [--download]

Requirements:
- mdbook installed
- mdbook-plantuml and mdbook-mermaid plugins installed (or use plugin binaries in PATH)
- Java + PlantUML jar and Graphviz installed for PlantUML rendering

Using the SW_DOC_Prompt with a coding agent and mdbook-cv
-----------------------------------------------------

1. Open documentation/SW_DOC_Prompt.md — this contains a ready-to-use prompt asking a coding agent to generate a C4-style documentation suite.
2. Run a coding agent (your choice) and feed it the prompt. Save the generated files into documentation/c4/ exactly as the prompt requests (e.g., 01_system_context.md, 01_system_context.puml, ...).
3. Verify the generated files exist under documentation/c4/.

Generate and serve the book locally
----------------------------------

# Build or install mdbook-cv locally
- cargo install --path .           # installs binary to $HOME/.cargo/bin/mdbook-cv
- OR cargo build --release         # build locally and use target/release/mdbook-cv

# Prepare environment
- Install mdbook and plugins: cargo install mdbook mdbook-plantuml mdbook-mermaid
- Ensure Graphviz (dot) and Java (JRE) are installed for PlantUML rendering, or run:
    mdbook-cv check --download     # prompts to download plantuml.jar to ~/.plantuml/plantuml.jar

# Create the book from the generated docs and serve
- mdbook-cv generate documentation/c4 --out book
- mdbook-cv build book            # builds static site
- mdbook-cv serve book            # serves at http://localhost:3000 by default

Notes
-----
- If publishing to crates.io, users can install with `cargo install mdbook-cv` once published.
- For headless CI, install plantuml.jar and graphviz on the runner or use the CI steps in .github/workflows/ci.yml.
