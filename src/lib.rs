#[cfg(feature = "limit")]
use std::sync::atomic::AtomicI32;

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[cfg(feature = "limit")]
static COUNTER: AtomicI32 = AtomicI32::new(0);

#[cfg(feature = "limit")]
static MAX_COUNT: Lazy<i32> = Lazy::new(|| {
    std::env::var("GAS_TRANSLATION_LIMIT")
        .ok()
        .map(|s| {
            s.parse().expect(&format!(
                "The value 'GAS_TRANSLATION_LIMIT={}' is invalid.",
                s
            ))
        })
        .unwrap_or(3)
});

static TRANSLATION_URL: Lazy<String> =
    Lazy::new(|| std::env::var("GAS_TRANSLATION_URL").expect("'GAS_TRANSLATION_URL' is not set."));

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

#[cfg(feature = "limit")]
pub async fn translate(text: &str, source: &str, target: &str) -> Result<String> {
    use std::{hint::spin_loop, sync::atomic::Ordering, time::Duration};

    while let Err(_) = COUNTER.fetch_update(Ordering::Relaxed, Ordering::Acquire, |current| {
        if current < *MAX_COUNT {
            Some(current + 1)
        } else {
            None
        }
    }) {
        tokio::time::sleep(Duration::from_millis(0)).await;
        spin_loop();
    }

    let result = translate_impl(text, source, target).await;
    COUNTER.fetch_sub(1, Ordering::Release);
    result
}

#[cfg(not(feature = "limit"))]
pub async fn translate(text: &str, source: &str, target: &str) -> Result<String> {
    translate_impl(text, source, target).await
}

async fn translate_impl(text: &str, source: &str, target: &str) -> Result<String> {
    let data = RequestParams {
        text,
        source,
        target,
    };
    let url = &TRANSLATION_URL[..];
    let client = reqwest::Client::new();
    let response = client.post(url).json(&data).send().await?;
    let result = response.json::<ApiResponse>().await?;
    if result.code == 200 {
        Ok(result.text)
    } else {
        anyhow::bail!("{}", result.text)
    }
}
