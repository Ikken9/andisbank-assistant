use std::fs;
use crate::client::client::{OpenAIClient};

mod client;

pub fn read_file_content(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        Ok(content)
    }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "api-key".to_string();
    let client = OpenAIClient::new(api_key);

    let file_path = "bank_policies.json";
    let file_content = read_file_content(file_path)?;
    let file_id = client.upload_document(file_path).await?;
    println!("File uploaded with ID: {}", file_id);

    let query = "What is the policy for credit cards?";
    let response = client.query_assistant_with_content(query, &file_content).await?;
    println!("Assistant Response: {}", response);

    Ok(())
}