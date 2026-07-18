mdbook-cv
===========

A small Rust CLI to generate an mdBook from a folder of Markdown files and support PlantUML (bundled jar) and Mermaid via mdbook plugins.

Usage (after building):

  mdbook-cv generate <input-dir> [--out <book-dir>]
  mdbook-cv build <book-dir>
  mdbook-cv serve <book-dir>

Requirements:
- mdbook installed
- mdbook-plantuml and mdbook-mermaid plugins installed (or use plugin binaries in PATH)
- Java + PlantUML jar and Graphviz installed for PlantUML rendering
