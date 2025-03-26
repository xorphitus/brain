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
    /// Truncates a string to a maximum number of characters, preserving Unicode character boundaries
    fn truncate_to_char_limit(text: &str, max_chars: usize) -> String {
        let char_count = text.chars().count();
        if char_count > max_chars {
            text.chars().take(max_chars).collect::<String>()
        } else {
            text.to_string()
        }
    }

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
        let truncated_context = Self::truncate_to_char_limit(context, self.max_context_length);

        let system = "You are a knowledge assistant that provides accurate information based on the given context. Only use the provided information to answer queries. Do not make up facts or use external knowledge. Your answer must be in the same language as the query.";
        
        let prompt = format!(
            "Use the following information to answer the query:\n\nINFORMATION:\n{}\n\nQUERY:\n{}\n\nANSWER:",
            truncated_context, query
        );

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
    fn test_truncate_to_char_limit() {
        let unicode_text = "こんにちは世界！これはテストです。".repeat(100);
        let max_length = 10;
        let truncated_context = OllamaClient::truncate_to_char_limit(&unicode_text, max_length);
        
        assert_eq!(truncated_context.chars().count(), max_length);        
        assert!(unicode_text.starts_with(&truncated_context));
        assert!(std::str::from_utf8(truncated_context.as_bytes()).is_ok());
    }

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
