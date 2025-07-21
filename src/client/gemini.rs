use log::{info, trace};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ClientRequest {
    contents: Vec<Content>,
    #[serde(rename = "systemInstruction")]
    system_instruction: Content,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct ClientResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: i32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: i32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: i32,
}

pub struct Stats {
    pub prompt_token_count: i32,
    pub candidates_token_count: i32,
    pub total_token_count: i32,
}

impl Stats {
    pub fn log(&self) {
        info!(
            "prompt tokens: {} candidates tokens: {} total tokens: {}",
            self.prompt_token_count, self.candidates_token_count, self.total_token_count
        );
    }
}

pub struct Response {
    pub stats: Stats,
    pub text: String,
}

pub async fn request(
    model: &str,
    api_key: &String,
    prompt: &str,
    user_contents: &Vec<String>,
) -> Result<Response, Error> {
    let client = Client::new();
    let request_body = to_request_body(prompt, user_contents);
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );
    let response = client.post(url).json(&request_body).send().await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }
    let response = response.unwrap();

    let status = response.status();
    let response_text = response.text().await.expect("API Response to Text error");
    let response_body: ClientResponse =
        serde_json::from_str(&response_text).expect("API Response to JSON error");

    if response_body.candidates.is_empty() {
        info!("response status: {status}");
        trace!("response error: {response_text}");
    }

    let text = response_body
        .candidates
        .first()
        .map_or("", |candidate| {
            &candidate.content.parts.first().unwrap().text
        })
        .to_string();
    let usage = response_body.usage_metadata.unwrap();

    tokio::time::sleep(Duration::from_secs(30)).await;

    Ok(Response {
        text,
        stats: Stats {
            total_token_count: usage.total_token_count,
            prompt_token_count: usage.prompt_token_count,
            candidates_token_count: usage.candidates_token_count,
        },
    })
}

fn to_request_body(prompt: &str, user_contents_text_vec: &Vec<String>) -> ClientRequest {
    let mut parts = vec![];
    for text in user_contents_text_vec {
        parts.push(Part { text: text.clone() });
    }

    ClientRequest {
        system_instruction: Content {
            parts: vec![Part {
                text: prompt.to_string(),
            }],
        },
        generation_config: GenerationConfig {
            response_mime_type: "application/json".to_string(),
        },
        contents: vec![Content { parts }],
    }
}
