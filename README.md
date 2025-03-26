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
- [use-package](https://github.com/jwiegley/use-package) (recommended)
- [Consult](https://github.com/minad/consult) package
- jq command-line tool

### Configuration with use-package

Add the following configuration to your Emacs init file (e.g., `~/.emacs.d/init.el` or `~/.config/emacs/init.el`):

```elisp
;; Ensure use-package is installed
(unless (package-installed-p 'use-package)
  (package-refresh-contents)
  (package-install 'use-package))

(require 'use-package)

;; Install and configure Consult (dependency for brain-search)
(use-package consult
  :ensure t
  :bind (("C-s" . consult-line)
         ("C-x b" . consult-buffer)
         ("M-y" . consult-yank-pop))
  :config
  (setq consult-preview-key "M-."))

;; Configure brain-search
(use-package brain-search
  :load-path "/path/to/brain/directory" ;; Update this path to where brain-search.el is located
  :after consult
  :bind (("C-c b" . brain-search))
  :custom
  (brain-search-command "brain") ;; Path to the brain command
  (brain-search-jq-command "jq") ;; Path to the jq command
  :config
  ;; Optional: Define a function to search and insert results at point
  (defun brain-search-insert ()
    "Search brain knowledge base and insert selected content at point."
    (interactive)
    (let* ((query (read-string "Brain search query for insertion: "))
           (shell-command (format "%s --mode generate-response --format text %s"
                                 brain-search-command
                                 (shell-quote-argument query)))
           (output (shell-command-to-string shell-command)))
      (when output
        (insert output))))
  
  ;; Bind the insert function to a key
  (global-set-key (kbd "C-c B") 'brain-search-insert))
```

### Customization Options

You can customize the configuration to suit your needs:

```elisp
(use-package brain-search
  :load-path "/path/to/brain/directory"
  :after consult
  :custom
  ;; Use a specific model for Emacs queries
  (brain-search-command "brain --model llama3") 
  
  ;; Increase max files for more comprehensive results
  (brain-search-command "brain --max-files 10")
  
  ;; Custom keybindings
  :bind
  (("C-c b" . brain-search)
   ("C-c B" . brain-search-insert))
  
  ;; Additional configuration
  :config
  ;; Integration with other Emacs packages
  (with-eval-after-load 'org
    (define-key org-mode-map (kbd "C-c b") 'brain-search)))
```

### Usage

1. Install the required packages using the configuration above
2. Run the command with `M-x brain-search` or the configured keybinding (e.g., `C-c b`)
3. Enter your query when prompted
4. Select a file from the results using Consult
5. Optionally use `brain-search-insert` (e.g., `C-c B`) to insert generated content directly at point
