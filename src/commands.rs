use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

const DEFAULT_BASE_URL: &str = "https://api.shopsavvy.com/v1";

impl Config {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn get(&self, path: &str, params: &[(&str, String)]) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "tauri-plugin-shopsavvy/0.1.0")
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(status.as_u16(), body));
        }
        Ok(resp.json::<Value>().await?)
    }
}

#[tauri::command]
pub async fn search_products(
    config: State<'_, Config>,
    query: String,
    limit: Option<u32>,
) -> Result<Value> {
    let mut params = vec![("q", query)];
    if let Some(l) = limit {
        params.push(("limit", l.to_string()));
    }
    config.get("/products/search", &params).await
}

#[tauri::command]
pub async fn get_offers(config: State<'_, Config>, identifier: String) -> Result<Value> {
    let params = vec![("id", identifier)];
    config.get("/products/offers", &params).await
}

#[tauri::command]
pub async fn get_price_history(
    config: State<'_, Config>,
    identifier: String,
    days: Option<u32>,
) -> Result<Value> {
    let mut params = vec![("id", identifier)];
    if let Some(d) = days {
        params.push(("days", d.to_string()));
    }
    config.get("/products/history", &params).await
}

#[tauri::command]
pub async fn get_deals(
    config: State<'_, Config>,
    options: Option<HashMap<String, Value>>,
) -> Result<Value> {
    let mut params = vec![("limit", "20".to_string()), ("sort", "trending".to_string())];
    if let Some(opts) = options {
        for (k, v) in opts {
            if let Some(s) = v.as_str() {
                params.push((Box::leak(k.into_boxed_str()), s.to_string()));
            } else if let Some(n) = v.as_i64() {
                params.push((Box::leak(k.into_boxed_str()), n.to_string()));
            }
        }
    }
    config.get("/deals", &params).await
}
