# Brain Knowledge System

A CLI tool for querying your knowledge base with LLM integration.

## Overview

Brain is a command-line tool that helps you search through your personal knowledge base using natural language queries. It leverages Ollama for extracting search terms and generating responses based on the content of your files.

```
┌─────────┐     ┌────────┐     ┌───────────────┐
│  Query  │────▶│ Brain  │────▶│ Extract Terms │
└─────────┘     └────────┘     └───────────────┘
                    │                  │
                    │                  ▼
┌─────────────┐     │           ┌─────────────┐
│  Response   │◀────┘           │ Search Files│
└─────────────┘                 └─────────────┘
       ▲                               │
       │                               ▼
┌─────────────┐                ┌─────────────┐
│  Generate   │◀───────────────│ File Content│
└─────────────┘                └─────────────┘
```

## Features

- Extract relevant search terms from natural language queries
- Search your knowledge base for files matching those terms
- Generate responses using Ollama with the content of matched files as context
- Emacs integration via `brain-search.el`
- Flexible output formats (text or JSON)
- Multiple operation modes for different use cases

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- [Ollama](https://ollama.ai/download) (must be running locally)

### From Source

```bash
# Clone the repository
git clone https://github.com/xorphitus/brain.git
cd brain

# Build and install
cargo install --path .
```

### Arch Linux

If you're using Arch Linux or an Arch-based distribution, you can install using the provided PKGBUILD:

```bash
git clone https://github.com/xorphitus/brain.git
cd brain
makepkg -si
```

## Configuration

Create a configuration file at `~/.config/brain/config.toml`:

```toml
[ollama]
endpoint = "http://localhost:11434"
model = "mistral"
max_context_length = 4096

[knowledge]
root_path = "/path/to/your/knowledge/base"
max_files = 5  # Maximum number of files to include in context
```

### Configuration Options

- `ollama.endpoint`: URL of your Ollama instance
- `ollama.model`: Ollama model to use (e.g., mistral, llama2, etc.)
- `ollama.max_context_length`: Maximum context length for the model
- `knowledge.root_path`: Root directory of your knowledge base files
- `knowledge.max_files`: Maximum number of files to include in the context

## Usage

### Basic Usage

```bash
brain "What are the key features of my project?"
```

### Operation Modes

```bash
# Extract search terms only
brain --mode extract-only "What are the key features of my project?"

# Search files only (don't generate a response)
brain --mode search-only "What are the key features of my project?"

# Complete workflow (default)
brain --mode generate-response "What are the key features of my project?"
```

### Output Formats

```bash
# Text output (default)
brain "What are the key features of my project?"

# JSON output
brain --format json "What are the key features of my project?"
```

### Other Options

```bash
# Override max files from config
brain --max-files 10 "What are the key features of my project?"

# Use a different config file
brain --config /path/to/config.toml "What are the key features of my project?"
```

## Emacs Integration

Brain includes an Emacs package for integration with [Consult](https://github.com/minad/consult).

### Requirements

- Emacs 27.1 or later
- Consult package
- jq command-line tool

### Usage

1. Load the package: `(require 'brain-search)`
2. Run the command: `M-x brain-search`
3. Enter your query when prompted
4. Select a file from the results using Consult
