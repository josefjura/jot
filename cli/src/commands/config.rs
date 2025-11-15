use crate::app_config::AppConfig;

pub fn config_cmd(config: AppConfig) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string_pretty(&config)?;
    println!("{}", json);

    Ok(())
}
