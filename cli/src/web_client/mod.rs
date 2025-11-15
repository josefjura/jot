use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app_config::AppConfig,
    args::NoteSearchArgs,
    model::{CreateNoteResponse, GetNotesResponse, TokenPollResponse},
};

#[cfg(debug_assertions)]
pub mod mock;

mod web;

// In your client code
#[cfg(debug_assertions)]
pub fn get_client(config: &AppConfig) -> Box<dyn Client> {
    use mock::MockClient;
    use web::WebClient;

    if config.mock_server {
        Box::new(MockClient::new())
    } else {
        Box::new(WebClient::new(
            config.server_url.clone(),
            config.token.clone(),
        ))
    }
}

#[cfg(not(debug_assertions))]
pub fn get_client(config: &AppConfig) -> Box<dyn Client> {
    use web::WebClient;

    Box::new(WebClient::new(config.server_url.clone()))
}

#[async_trait]
pub trait Client {
    async fn ping(&self) -> anyhow::Result<()>;
    async fn send_device_code(&self, device_code: &str) -> anyhow::Result<()>;
    async fn poll_for_token(&mut self, device_code: &str) -> anyhow::Result<TokenPollResponse>;
    async fn create_note(
        &mut self,
        content: String,
        tags: Vec<String>,
        date: NaiveDate,
    ) -> anyhow::Result<CreateNoteResponse>;
    async fn get_notes(&mut self) -> anyhow::Result<GetNotesResponse>;
    async fn search(&mut self, args: &NoteSearchArgs) -> anyhow::Result<GetNotesResponse>;
    fn get_server_url(&self) -> String;
}
