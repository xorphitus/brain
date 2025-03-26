use anyhow::{Context, Result};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use url::Url;

pub struct OllamaClient {
    client: Ollama,
    model: String,
    max_context_length: usize,
}

impl OllamaClient {
    pub fn new(endpoint: &str, model: &str, max_context_length: usize) -> Result<Self> {
        let endpoint_with_protocol =
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                format!("http://{}", endpoint)
            } else {
                endpoint.to_string()
            };

        let url = Url::parse(&endpoint_with_protocol)
            .with_context(|| format!("Invalid endpoint URL: {}", endpoint))?;

        let url_str = url.to_string();
        let client = Ollama::try_new(url.clone())
            .with_context(|| format!("Failed to create Ollama client with URL: {}", url_str))?;

        Ok(Self {
            client,
            model: model.to_string(),
            max_context_length,
        })
    }

    /// Extracts search terms from a user query using Ollama
    /// This includes both direct terms from the query and related/recalled terms
    pub async fn extract_search_terms(&self, query: &str) -> Result<Vec<String>> {
        let system = "You are a search term extraction assistant. Your task is to analyze queries and extract useful search terms. You can detect the language of queries. For non-English queries, you provide terms in both the original language and English translations. For English queries, you provide terms in English only.";
        let prompt = format!(
            "Extract the most important search terms from this query. Include both direct terms and related/recalled terms that would be useful for searching a knowledge base. Return only the terms, one per line, with no additional text or explanation:\n\n{}",
            query
        );

        let request = GenerationRequest::new(self.model.clone(), prompt)
            .system(system);
            
        let response = self
            .client
            .generate(request)
            .await
            .context("Failed to extract search terms using Ollama")?;

        let terms: Vec<String> = response
            .response
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(terms)
    }

    /// Generates a response based on the query and context
    pub async fn generate_response(&self, query: &str, context: &str) -> Result<String> {
        // Truncate context if it's too long
        let context = if context.len() > self.max_context_length {
            &context[..self.max_context_length]
        } else {
            context
        };

        // System prompt defines the role and capabilities
        let system = "You are a knowledge assistant that provides accurate information based on the given context. Only use the provided information to answer queries. Do not make up facts or use external knowledge.";
        
        // User prompt contains the specific task for this query
        let prompt = format!(
            "Use the following information to answer the query:\n\nINFORMATION:\n{}\n\nQUERY:\n{}\n\nANSWER:",
            context, query
        );

        // Create request with system and user prompts
        let request = GenerationRequest::new(self.model.clone(), prompt)
            .system(system);
            
        let response = self
            .client
            .generate(request)
            .await
            .context("Failed to generate response using Ollama")?;

        Ok(response.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_valid_url_with_protocol() {
        let result = OllamaClient::new("http://localhost:11434", "model", 4096);
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_without_protocol() {
        let result = OllamaClient::new("localhost:11434", "model", 4096);
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_without_port() {
        let result = OllamaClient::new("localhost", "model", 4096);
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_with_trailing_slash() {
        let result = OllamaClient::new("http://localhost:11434/", "model", 4096);
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_invalid_url() {
        let result = OllamaClient::new("invalid:url:format", "model", 4096);
        assert!(result.is_err());
    }
}
