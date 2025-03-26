use anyhow::{Context, Result};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

pub struct OllamaClient {
    client: Ollama,
    model: String,
    max_context_length: usize,
}

impl OllamaClient {
    pub fn new(endpoint: &str, model: &str, max_context_length: usize) -> Self {
        // Parse endpoint to extract host and port
        let endpoint = endpoint.trim_end_matches('/');
        
        // Remove protocol if present
        let host_port = if endpoint.starts_with("http://") {
            &endpoint[7..]
        } else if endpoint.starts_with("https://") {
            &endpoint[8..]
        } else {
            endpoint
        };
        
        // Split host and port
        let parts: Vec<&str> = host_port.split(':').collect();
        let host = parts[0].to_string();
        let port = if parts.len() > 1 {
            parts[1].parse().unwrap_or(11434)
        } else {
            11434 // Default Ollama port
        };
        
        let client = Ollama::new(host, port);
        
        Self {
            client,
            model: model.to_string(),
            max_context_length,
        }
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
    // Tests would be added here, but they would require mocking the Ollama API
    // which is beyond the scope of this implementation
}
