use crate::{
    app_config::AppConfig,
    auth::AuthFlow,
    web_client::{self, Client},
};

pub async fn login_cmd(
    mut client: Box<dyn Client>,
    profile_path: Option<&str>,
    api_key_path: &str,
) -> Result<(), anyhow::Error> {
    if let Some(profile_path) = profile_path {
        println!("Using profile: {:?}", profile_path);
    }
    let token = AuthFlow::new().login(client.as_mut()).await;

    match token {
        Ok(token) => {
            println!("Api Key Path: {}", api_key_path);
            std::fs::write(api_key_path, token)?;
            println!("User successfully logged in.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
