mod config;
mod search;
mod content;
mod ollama;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;


use crate::config::{load_config, load_config_from_path};
use crate::content::get_contents;
use crate::ollama::OllamaClient;
use crate::search::search_files;

/// Operation mode for the brain tool
#[derive(ValueEnum, Clone, Debug)]
enum Mode {
    /// Only extract and display search terms
    ExtractOnly,
    /// Extract terms and find matching files
    SearchOnly,
    /// Complete workflow including response generation
    GenerateResponse,
}

/// Output format for the brain tool
#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// Standard text output
    Text,
    /// JSON formatted output
    Json,
}

/// Response structure for JSON output
#[derive(Serialize)]
struct BrainResponse {
    query: String,
    search_terms: Vec<String>,
    matched_files: Vec<search::SearchResult>,
    response: String,
}

/// Brain Knowledge System - A CLI tool for querying your knowledge base
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The query to process
    #[clap(required = true)]
    query: String,
    
    /// Operation mode: extract-only, search-only, or generate-response
    #[clap(long, value_enum, default_value_t = Mode::GenerateResponse)]
    mode: Mode,
    
    /// Output format: text or json
    #[clap(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    
    /// Override the maximum number of files to use
    #[clap(long)]
    max_files: Option<usize>,
    
    /// Specify an alternative config file path
    #[clap(long, value_parser)]
    config: Option<PathBuf>,
}

async fn run() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();
    
    // Load configuration
    let mut config = match &args.config {
        Some(config_path) => load_config_from_path(config_path)?,
        None => load_config()?,
    };
    
    // Override max_files if specified in CLI args
    if let Some(max_files) = args.max_files {
        config.knowledge.max_files = max_files;
    }
    
    // Initialize Ollama client
    let ollama_client = OllamaClient::new(
        &config.ollama.endpoint,
        &config.ollama.model,
        config.ollama.max_context_length,
    )?;
    
    // Extract search terms from query
    if matches!(args.format, OutputFormat::Text) {
        println!("Extracting search terms from query...");
    }
    let search_terms = ollama_client.extract_search_terms(&args.query).await?;
    
    if matches!(args.format, OutputFormat::Text) {
        println!("Search terms: {:?}", search_terms);
    }
    
    // If extract_only mode, output and stop here
    if matches!(args.mode, Mode::ExtractOnly) {
        if matches!(args.format, OutputFormat::Json) {
            let response = BrainResponse {
                query: args.query.clone(),
                search_terms,
                matched_files: vec![],
                response: String::new(),
            };
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        return Ok(());
    }
    
    // Search files based on search terms
    if matches!(args.format, OutputFormat::Text) {
        println!("Searching files...");
    }
    let search_results = search_files(&config, &search_terms)?;
    
    if search_results.is_empty() && matches!(args.format, OutputFormat::Text) {
        println!("No matching files found.");
    }
    
    // Display search results in text mode
    if matches!(args.format, OutputFormat::Text) {
        println!("\nFound {} matching files:", search_results.len());
        for (i, result) in search_results.iter().enumerate() {
            println!("{}. {} (relevance: {:.2})", i + 1, result.path, result.relevance);
        }
    }
    
    // If search_only mode, output and stop here
    if matches!(args.mode, Mode::SearchOnly) {
        if matches!(args.format, OutputFormat::Json) {
            let response = BrainResponse {
                query: args.query.clone(),
                search_terms,
                matched_files: search_results,
                response: String::new(),
            };
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        return Ok(());
    }
    
    // Get file paths from search results
    let file_paths: Vec<String> = search_results.iter()
        .map(|r| r.path.clone())
        .collect();
    
    // Retrieve file contents
    if matches!(args.format, OutputFormat::Text) {
        println!("\nRetrieving file contents...");
    }
    let contents = get_contents(&file_paths)?;
    
    // Generate response using Ollama
    if matches!(args.format, OutputFormat::Text) {
        println!("\nGenerating response...");
    }
    let response = ollama_client.generate_response(&args.query, &contents).await?;
    
    // Output the final result
    if matches!(args.format, OutputFormat::Text) {
        println!("\nResponse:");
        println!("{}", response);
    } else {
        // JSON output
        let brain_response = BrainResponse {
            query: args.query.clone(),
            search_terms,
            matched_files: search_results,
            response,
        };
        println!("{}", serde_json::to_string_pretty(&brain_response)?);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        
        // Print cause chain for better error diagnostics
        let mut cause = e.source();
        while let Some(e) = cause {
            eprintln!("Caused by: {}", e);
            cause = e.source();
        }
        
        std::process::exit(1);
    }
    
    Ok(())
}
