use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::fs::File;
use serde_json;
use ureq;
use std::process::Command;

// struct to read config file for HF token
#[derive(Debug, Deserialize)]
struct Config {
    token: String,
}

// main struct for summary
#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    video_id: Option<String>,
    transcript: Option<String>,
    summary: Option<String>,
}

// struct to store api response from HF transformer model
#[derive(Debug, Deserialize)]
struct ApiResponse {
    summary_text: String,
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
        println!("Fetching transcript for video ID: {}", video_id);
        
        let output = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "from youtube_transcript_api import YouTubeTranscriptApi; \
                 transcript = YouTubeTranscriptApi.get_transcript('{}'); \
                 print(' '.join(entry['text'] for entry in transcript))",
                video_id
            ))
            .output()
            .context("Failed to execute python command. Make sure python3 and youtube_transcript_api are installed")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to get transcript: {}", error));
        }

        let transcript = String::from_utf8(output.stdout)
            .context("Failed to parse transcript output")?;

        Ok(transcript)
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
        
        println!("Processing {} chunks...", chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            println!("Summarizing chunk {}/{}...", i + 1, chunks.len());
            
            let response = ureq::post(&self.api_url)
                .set("Authorization", &format!("Bearer {}", self.api_token))
                .send_json(ureq::json!({
                    "inputs": chunk,
                    "parameters": {
                        "max_length": 150,
                        "min_length": 30,
                        "do_sample": false
                    }
                }))
                .context("Failed to send request to API")?;

            let summary: Vec<ApiResponse> = response.into_json()
                .context("Failed to parse API response")?;
            
            if let Some(first_summary) = summary.first() {
                summaries.push(first_summary.summary_text.clone());
            }
        }

        if summaries.is_empty() {
            return Err(anyhow::anyhow!("No summaries were generated"));
        }

        // Join all summaries with newlines between them
        Ok(summaries.join("\n\n"))
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

        let transcript = Self::get_transcript(&video_id)?;
        result.transcript = Some(transcript.clone());
        
        let summary = self.summarize_text(&transcript)?;
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
        Err(e) => println!("Error: {:#}", e),
    }

    Ok(())
}