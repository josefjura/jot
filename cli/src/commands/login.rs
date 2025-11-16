use crate::{
    app_config::AppConfig,
    auth::AuthFlow,
    web_client::{self, Client},
};
use std::fs;
use std::path::Path;

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
            save_token_securely(api_key_path, &token)?;
            println!("User successfully logged in.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn save_token_securely(token_path: &str, token: &str) -> anyhow::Result<()> {
    let path = Path::new(token_path);

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the token
    fs::write(path, token)?;

    // On Unix-like systems, set file permissions to 600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }

    Ok(())
}
