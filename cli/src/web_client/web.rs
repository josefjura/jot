use async_trait::async_trait;
use chrono::NaiveDate;
use serde_json::json;

use crate::{
    args::NoteSearchArgs,
    model::{
        CreateNoteResponse, DeviceCodeRequest, GetNotesResponse, Note, Token, TokenPollResponse,
    },
};

use super::Client;

pub struct WebClient {
    server_url: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl WebClient {
    pub fn new(server_url: String, token: Option<String>) -> Self {
        Self {
            server_url,
            token,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Client for WebClient {
    async fn send_device_code(&self, device_code: &str) -> anyhow::Result<()> {
        let data = DeviceCodeRequest {
            device_code: device_code.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/auth/device", self.server_url))
            .json(&data)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to register device");
        }

        Ok(())
    }

    async fn poll_for_token(&mut self, device_code: &str) -> anyhow::Result<TokenPollResponse> {
        let response = self
            .client
            .get(format!("{}/auth/status/{}", self.server_url, device_code))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let token: Token = response.json().await?;
                Ok(TokenPollResponse::Success(token.access_token))
            }
            reqwest::StatusCode::ACCEPTED => Ok(TokenPollResponse::Pending),
            _ => anyhow::bail!("Authentication polling failed: {}", response.status()),
        }
    }

    async fn ping(&self) -> anyhow::Result<()> {
        let real_token = match self.token {
            Some(ref token) => token,
            None => anyhow::bail!("No token available"),
        };

        let response = self
            .client
            .get(format!("{}/health/auth", self.server_url))
            .header("Authorization", format!("Bearer {}", real_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Cannot verify login");
        }

        Ok(())
    }

    async fn create_note(
        &mut self,
        content: String,
        tags: Vec<String>,
        date: NaiveDate,
    ) -> anyhow::Result<CreateNoteResponse> {
        let real_token = match self.token {
            Some(ref token) => token,
            None => anyhow::bail!("No token available"),
        };

        let response = self
            .client
            .post(format!("{}/note", self.server_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", real_token))
            .json(&json!({
                "content": content,
                "tags": tags,
                "date": date
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            println!("{:?}", response.text().await);
            anyhow::bail!("Failed to create note");
        }

        let json = response.json::<CreateNoteResponse>().await?;

        Ok(CreateNoteResponse {
            id: json.id,
            content: json.content,
        })
    }

    async fn get_notes(&mut self) -> anyhow::Result<GetNotesResponse> {
        let real_token = match self.token {
            Some(ref token) => token,
            None => anyhow::bail!("No token available"),
        };

        let response = self
            .client
            .get(format!("{}/note", self.server_url))
            .header("Authorization", format!("Bearer {}", real_token))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get notes");
        }

        // DEBUG RESPONSE
        // let bytes = response.bytes().await?;
        // println!("{:?}", std::str::from_utf8(&bytes)?);

        // let notes: GetNotesResponse = serde_json::from_slice(&bytes)?;
        let notes = response.json::<Vec<Note>>().await?;

        Ok(GetNotesResponse { notes })
    }

    async fn search(&mut self, args: &NoteSearchArgs) -> anyhow::Result<GetNotesResponse> {
        let real_token = match self.token {
            Some(ref token) => token,
            None => anyhow::bail!("No token available"),
        };

        let response = self
            .client
            .post(format!("{}/note/search", self.server_url))
            .json(&args)
            .bearer_auth(real_token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to seaarch for notes");
        }

        let notes = response.json::<Vec<Note>>().await?;

        Ok(GetNotesResponse { notes })
    }

    fn get_server_url(&self) -> String {
        self.server_url.clone()
    }
}
