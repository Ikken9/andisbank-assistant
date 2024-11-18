use reqwest::{Client, Error, multipart};
use serde_json::Value;
use thiserror::Error;
use std::fs;
use std::path::Path;

pub struct OpenAIClient {
    api_key: String,
    client: Client,
}

#[derive(Error, Debug)]
        pub enum UploadError {
        #[error("File read error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Request error: {0}")]
        Reqwest(#[from] reqwest::Error),
    }


impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    async fn post_request(&self, url: &str, body: Value) -> Result<Value, Error> {
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    pub async fn upload_document(&self, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/files";

        let file_name = Path::new(file_path)
            .file_name()
            .ok_or("Invalid file path")?
            .to_string_lossy()
            .to_string();

        let file_content = fs::read(file_path)?;

        let form = multipart::Form::new()
            .text("purpose", "assistants")
            .part(
                "file",
                multipart::Part::bytes(file_content)
                    .file_name(file_name)
                    .mime_str("application/octet-stream")?,
            );

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;

        if let Some(file_id) = response_body.get("id").and_then(|id| id.as_str()) {
            Ok(file_id.to_string())
        } else {
            Err(format!(
                "Failed to retrieve file ID from response: {:?}",
                response_body
            )
            .into())
        }
    }

    pub async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, Error> {
        let url = "https://api.openai.com/v1/embeddings";

        let body = serde_json::json!({
            "model": "text-embedding-ada-002",
            "input": text,
        });

        let response = self.post_request(url, body).await?;
        let embeddings = response["data"][0]["embedding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();

        Ok(embeddings)
    }

    pub fn read_file_content(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        Ok(content)
    }

    pub async fn query_assistant_with_content(
    &self,
    query: &str,
    file_content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.openai.com/v1/chat/completions";

        let prompt = format!(
            "Based on the following content:\n\n{}\n\nQuery: {}\nAssistant Response:",
            file_content, query
        );

        let body = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are an assistant that uses provided content to answer queries."},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 200,
            "temperature": 0.7,
        });

        let response: serde_json::Value = self.post_request(url, body).await?;
        if let Some(answer) = response["choices"]
            .get(0)
            .and_then(|choice| choice["message"]["content"].as_str())
        {
            Ok(answer.to_string())
        } else {
            Err("Failed to retrieve a valid response from the assistant.".into())
        }
    }

}
