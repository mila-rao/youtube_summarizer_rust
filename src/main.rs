use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::fs::File;
use serde_json;
use ureq;

// Response struct for transcript
#[derive(Debug, Deserialize)]
struct TranscriptResponse {
    text: String,
}

// Config struct for reading token
#[derive(Debug, Deserialize)]
struct Config {
    token: String,
}

// Main summary struct
#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    video_id: Option<String>,
    transcript: Option<String>,
    summary: Option<String>,
}

struct HuggingFaceSummarizer {
    api_token: String,
    api_url: String,
}

impl HuggingFaceSummarizer {
    fn new(api_token: String) -> Self {
        HuggingFaceSummarizer {
            api_token,
            api_url: "https://api-inference.huggingface.co/models/facebook/bart-large-cnn".to_string(),
        }
    }

    fn extract_video_id(youtube_url: &str) -> Option<String> {
        let re = Regex::new(r"(?:v=|/)([0-9A-Za-z_-]{11}).*").unwrap();
        re.captures(youtube_url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn get_transcript(video_id: &str) -> Result<String> {
        let url = format!(
            "url",
            video_id
        );
        
        println!("Fetching transcript from: {}", url);
        
        let response = ureq::get(&url)
            .call()
            .context("Failed to fetch transcript")?;
        
        // Print the raw response for debugging
        let response_text = response.into_string()?;
        println!("Raw API Response: {}", response_text);
        
        // Try to parse the response
        let transcript_entries: Vec<TranscriptResponse> = serde_json::from_str(&response_text)
            .context(format!("Failed to parse transcript JSON: {}", response_text))?;

        Ok(transcript_entries
            .into_iter()
            .map(|entry| entry.text)
            .collect::<Vec<String>>()
            .join(" "))
    }

    fn chunk_text(text: &str, max_length: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_length = 0;

        for word in words {
            let word_len = word.len() + 1;
            if current_length + word_len > max_length {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.join(" "));
                    current_chunk.clear();
                    current_length = 0;
                }
            }
            current_chunk.push(word);
            current_length += word_len;
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join(" "));
        }

        chunks
    }

    fn summarize_text(&self, text: &str) -> Result<String> {
        let chunks = Self::chunk_text(text, 1024);
        let mut summaries = Vec::new();

        for chunk in chunks {
            let response = ureq::post(&self.api_url)
                .set("Authorization", &format!("Bearer {}", self.api_token))
                .send_json(ureq::json!({
                    "inputs": chunk,
                    "parameters": {
                        "max_length": 150,
                        "min_length": 30,
                        "do_sample": false
                    }
                }))?;

            let summary: Vec<String> = response.into_json()?;
            if let Some(first_summary) = summary.first() {
                summaries.push(first_summary.clone());
            }
        }

        summaries
            .first()
            .cloned()
            .context("No summary generated")
    }

    fn process_video(&self, youtube_url: &str) -> Result<Summary> {
        let video_id = Self::extract_video_id(youtube_url)
            .context("Failed to extract video ID from URL")?;
        
        println!("Extracted video ID: {}", video_id);
        
        let mut result = Summary {
            video_id: Some(video_id.clone()),
            transcript: None,
            summary: None,
        };

        let transcript = Self::get_transcript(&video_id)
            .context("Failed to fetch transcript")?;
        result.transcript = Some(transcript.clone());
        
        let summary = self.summarize_text(&transcript)
            .context("Failed to generate summary")?;
        result.summary = Some(summary);

        Ok(result)
    }
}

fn read_config(path: &str) -> Result<String> {
    let file = File::open(path)
        .context(format!("Failed to open config file at {}", path))?;
    
    let config: Config = serde_json::from_reader(file)
        .context("Failed to parse config file")?;
    
    Ok(config.token)
}

fn main() -> Result<()> {
    let config_path = "config.json";
    
    let api_token = read_config(config_path)
        .context("Failed to read API token from config")?;

    let summarizer = HuggingFaceSummarizer::new(api_token);

    print!("Enter YouTube video URL: ");
    io::stdout().flush()?;

    let mut youtube_url = String::new();
    io::stdin().read_line(&mut youtube_url)?;

    match summarizer.process_video(&youtube_url) {
        Ok(result) => {
            if let Some(summary) = result.summary {
                println!("\nVideo Summary:");
                println!("{}", "-".repeat(50));
                println!("{}", summary);
            } else {
                println!("Failed to generate summary");
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}