use std::{fs, path::PathBuf, time::Instant};

use rand::Rng;
use std::time::Duration;

use crate::{
    model::{Token, TokenPollResponse},
    web_client::Client,
};

const POLLING_INTERVAL: Duration = Duration::from_secs(3);
const MAX_POLLING_DURATION: Duration = Duration::from_secs(180); // 3 minutes

pub struct AuthFlow {}

impl AuthFlow {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn login(&self, client: &mut dyn Client) -> anyhow::Result<String> {
        // Generate a secure random device code
        let device_code = self.generate_device_code();

        println!("Sending device code: {}", device_code);

        // Register the device code with the server
        self.register_device(client, &device_code).await?;

        // Generate and open the authentication URL
        let auth_url = format!("{}/auth/page/{}", client.get_server_url(), device_code);
        println!(
            "Browser window should open, if not, please visit following URL to login: {}",
            auth_url
        );

        #[cfg(not(test))]
        webbrowser::open(&auth_url)?;

        // Poll for completion
        let token = self.poll_for_token(client, &device_code).await?;

        Ok(token)
    }

    fn generate_device_code(&self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";
        const CODE_LENGTH: usize = 32;

        let mut rng = rand::thread_rng();
        (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    async fn register_device(&self, client: &dyn Client, device_code: &str) -> anyhow::Result<()> {
        client.send_device_code(device_code).await?;

        Ok(())
    }

    async fn poll_for_token(
        &self,
        client: &mut dyn Client,
        device_code: &str,
    ) -> anyhow::Result<String> {
        let start_time = Instant::now();

        while start_time.elapsed() < MAX_POLLING_DURATION {
            let response = client.poll_for_token(device_code).await?;

            match response {
                TokenPollResponse::Pending => {
                    tokio::time::sleep(POLLING_INTERVAL).await;
                }
                TokenPollResponse::Success(token) => {
                    return Ok(token);
                }
                TokenPollResponse::Failure(message) => {
                    anyhow::bail!("Authentication failed: {}", message);
                }
            }
        }

        anyhow::bail!("Authentication timed out")
    }

    async fn check_auth(&self, client: &mut dyn Client) -> anyhow::Result<()> {
        client.ping().await?;

        Ok(())
    }

    #[expect(dead_code)]
    fn save_token(&self, token_path: &PathBuf, token: &str) -> anyhow::Result<()> {
        // Ensure the directory exists
        if let Some(parent) = token_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Save the token
        let token_data = Token {
            access_token: token.to_string(),
        };
        let token_json = serde_json::to_string_pretty(&token_data)?;
        fs::write(token_path, token_json)?;

        // On Unix-like systems, set file permissions to 600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(token_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(token_path, perms)?;
        }

        Ok(())
    }
}
