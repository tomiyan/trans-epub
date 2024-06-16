use log::{debug, info, trace};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ClientRequest {
    model: String,
    response_format: ResponseFormat,
    messages: Vec<MessageRequest>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    _type: String,
}

#[derive(Serialize)]
struct MessageRequest {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    #[serde(rename = "type")]
    _type: String,
    text: String,
}

#[derive(Deserialize)]
struct ClientResponse {
    #[serde(default)]
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

pub struct Stats {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

impl Stats {
    pub fn log(&self) {
        info!(
            "prompt tokens: {} completion tokens: {} total tokens: {}",
            self.prompt_tokens, self.completion_tokens, self.total_tokens
        );
    }
}

pub struct Ratelimit {
    pub limit_requests: String,
    pub limit_tokens: String,
    pub remaining_requests: String,
    pub remaining_tokens: String,
    pub reset_requests: String,
    pub reset_tokens: String,
}

impl Ratelimit {
    pub fn log(&self) {
        debug!("ratelimit limit requests: {}", self.limit_requests);
        debug!("ratelimit limit tokens: {}", self.limit_tokens);
        debug!("ratelimit remaining requests: {}", self.remaining_requests);
        debug!("ratelimit remaining tokens: {}", self.remaining_tokens);
        debug!("ratelimit reset requests: {}", self.reset_requests);
        debug!("ratelimit reset tokens: {}", self.reset_tokens);
    }

    pub fn reset_tokens_duration(&self) -> Duration {
        // TODO: add format ex 6m0s
        if let Some(milliseconds_str) = self.reset_tokens.strip_suffix("ms") {
            if let Ok(milliseconds) = milliseconds_str.parse::<u64>() {
                return Duration::from_millis(milliseconds);
            }
        } else if let Some(seconds_str) = self.reset_tokens.strip_suffix('s') {
            if let Ok(seconds) = seconds_str.parse::<f64>() {
                return Duration::from_secs_f64(seconds);
            }
        }
        Duration::from_secs(1)
    }
}

pub struct Response {
    pub stats: Stats,
    pub choice: String,
    pub ratelimit: Ratelimit,
}

pub async fn request(
    model: &str,
    api_key: &String,
    prompt: &str,
    user_contents: &Vec<String>,
) -> Result<Response, Error> {
    let client = Client::new();
    let request_body = to_request_body(model, prompt, user_contents);
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }
    let response = response.unwrap();
    let headers = response.headers();
    let ratelimit = Ratelimit {
        limit_requests: headers
            .get("x-ratelimit-limit-requests")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        limit_tokens: headers
            .get("x-ratelimit-limit-tokens")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        remaining_requests: headers
            .get("x-ratelimit-remaining-requests")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        remaining_tokens: headers
            .get("x-ratelimit-remaining-tokens")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        reset_requests: headers
            .get("x-ratelimit-reset-requests")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        reset_tokens: headers
            .get("x-ratelimit-reset-tokens")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    };

    let status = response.status();
    let response_text = response.text().await.expect("API Response to Text error");
    let response_body: ClientResponse =
        serde_json::from_str(&response_text).expect("API Response to JSON error");

    if response_body.choices.is_empty() {
        info!("response status: {}", status.to_string());
        trace!("response error: {}", response_text);
    }

    debug!(
        "sleep: {}sec",
        ratelimit.reset_tokens_duration().as_secs_f64()
    );
    tokio::time::sleep(ratelimit.reset_tokens_duration()).await;

    let choice = response_body
        .choices
        .first()
        .map_or("", |choice| &choice.message.content)
        .to_string();
    let usage = response_body.usage.unwrap();
    Ok(Response {
        choice,
        ratelimit,
        stats: Stats {
            total_tokens: usage.total_tokens,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
        },
    })
}

fn to_request_body(
    model: &str,
    prompt: &str,
    user_contents_text_vec: &Vec<String>,
) -> ClientRequest {
    let mut user_contents = vec![];
    for text in user_contents_text_vec {
        user_contents.push(Content {
            _type: "text".to_string(),
            text: text.clone(),
        });
    }

    ClientRequest {
        model: model.to_owned(),
        response_format: ResponseFormat {
            _type: "json_object".to_string(),
        },
        messages: vec![
            MessageRequest {
                role: "system".to_string(),
                content: vec![Content {
                    _type: "text".to_string(),
                    text: prompt.to_owned(),
                }],
            },
            MessageRequest {
                role: "user".to_string(),
                content: user_contents,
            },
        ],
    }
}
