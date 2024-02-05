use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    translation_url: String,
}

static CONFIG: Lazy<Config> = Lazy::new(|| envy::prefixed("GAS_").from_env().unwrap());

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    code: u32,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestParams<'a> {
    text: &'a str,
    source: &'a str,
    target: &'a str,
}

pub async fn translate(text: &str, source: &str, target: &str) -> Result<String> {
    let data = RequestParams {
        text,
        source,
        target,
    };
    let url = &CONFIG.translation_url;
    let client = reqwest::Client::new();
    let response = client.post(url).json(&data).send().await?;
    let result = response.json::<ApiResponse>().await?;
    if result.code == 200 {
        Ok(result.text)
    } else {
        anyhow::bail!("{}", result.text)
    }
}
