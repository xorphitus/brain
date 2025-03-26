use anyhow::{Context, Result};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use url::Url;

pub struct OllamaClient {
    client: Ollama,
    model: String,
    max_context_length: usize,
}

impl OllamaClient {
    pub fn new(endpoint: &str, model: &str, max_context_length: usize) -> Result<Self> {
        let endpoint_with_protocol = if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            format!("http://{}", endpoint)
        } else {
            endpoint.to_string()
        };
        
        let url = Url::parse(&endpoint_with_protocol)
            .with_context(|| format!("Invalid endpoint URL: {}", endpoint))?;
        
        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(11434);
        
        let client = Ollama::new(host, port);
        
        Ok(Self {
            client,
            model: model.to_string(),
            max_context_length,
        })
    }
    
    /// Extracts keywords from a user query using Ollama
    pub async fn extract_keywords(&self, query: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Extract the most important keywords from this query. Return only the keywords, one per line, with no additional text or explanation:\n\n{}",
            query
        );
        
        let request = GenerationRequest::new(self.model.clone(), prompt);
        let response = self.client.generate(request)
            .await
            .context("Failed to extract keywords using Ollama")?;
        
        // Parse the response to extract keywords
        let keywords: Vec<String> = response.response
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        
        Ok(keywords)
    }
    
    /// Generates a response based on the query and context
    pub async fn generate_response(&self, query: &str, context: &str) -> Result<String> {
        // Truncate context if it's too long
        let context = if context.len() > self.max_context_length {
            &context[..self.max_context_length]
        } else {
            context
        };
        
        let prompt = format!(
            "Use the following information to answer the query. Only use the provided information and don't make up facts.\n\nINFORMATION:\n{}\n\nQUERY:\n{}\n\nANSWER:",
            context, query
        );
        
        let request = GenerationRequest::new(self.model.clone(), prompt);
        let response = self.client.generate(request)
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
        let result = OllamaClient::new(
            "http://localhost:11434",
            "model",
            4096,
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_without_protocol() {
        let result = OllamaClient::new(
            "localhost:11434",
            "model",
            4096,
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_without_port() {
        let result = OllamaClient::new(
            "localhost",
            "model",
            4096,
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_valid_url_with_trailing_slash() {
        let result = OllamaClient::new(
            "http://localhost:11434/",
            "model",
            4096,
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.model, "model");
        assert_eq!(client.max_context_length, 4096);
    }

    #[test]
    fn test_new_with_invalid_url() {
        let result = OllamaClient::new(
            "invalid:url:format",
            "model",
            4096,
        );
        assert!(result.is_err());
    }
}
