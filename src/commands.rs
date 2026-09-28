use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.shopsavvy.com/v1";

/// Days of history returned by `get_price_history` when `days` is omitted.
pub(crate) const DEFAULT_HISTORY_DAYS: u32 = 30;

/// Query parameters, owned so option keys never have to outlive the call.
pub(crate) type Params = Vec<(String, String)>;

impl Config {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn get(&self, path: &str, params: &Params) -> Result<Value> {
        if self.api_key.trim().is_empty() {
            return Err(Error::MissingApiKey);
        }

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .bearer_auth(&self.api_key)
            .header(
                reqwest::header::USER_AGENT,
                concat!("tauri-plugin-shopsavvy/", env!("CARGO_PKG_VERSION")),
            )
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

/// GET /products/search?q=&limit=
pub(crate) fn search_params(query: String, limit: Option<u32>) -> Params {
    let mut params = vec![("q".to_string(), query)];
    if let Some(l) = limit {
        params.push(("limit".to_string(), l.to_string()));
    }
    params
}

/// GET /products/offers?ids= — the API reads `ids` (one identifier or a
/// comma-separated list); an `id` parameter is rejected with a 400.
pub(crate) fn offers_params(identifier: String) -> Params {
    vec![("ids".to_string(), identifier)]
}

/// GET /products/offers/history?ids=&start=&end= — `start` and `end` are
/// required YYYY-MM-DD dates. `days` (default 30) is the window ending today (UTC).
pub(crate) fn price_history_params(identifier: String, days: Option<u32>, today: (i64, u32, u32)) -> Params {
    let days = days.unwrap_or(DEFAULT_HISTORY_DAYS);
    let end_days = days_from_civil(today.0, today.1, today.2);
    let start = civil_from_days(end_days - i64::from(days));
    vec![
        ("ids".to_string(), identifier),
        ("start".to_string(), format_date(start)),
        ("end".to_string(), format_date(today)),
    ]
}

/// GET /deals — every option is passed through as a query parameter. `sort`
/// defaults to `hot` and `limit` to 20 only when the caller did not set them
/// (sending both a default and the caller's value produced `limit=20&limit=8`,
/// and the API read the first).
pub(crate) fn deals_params(options: Option<HashMap<String, Value>>) -> Params {
    let mut params: Params = Vec::new();
    let options = options.unwrap_or_default();

    let mut keys: Vec<&String> = options.keys().collect();
    keys.sort();
    for key in keys {
        let value = match &options[key] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => continue,
            other => other.to_string(),
        };
        params.push((key.clone(), value));
    }

    if !options.contains_key("sort") {
        params.push(("sort".to_string(), "hot".to_string()));
    }
    if !options.contains_key("limit") {
        params.push(("limit".to_string(), "20".to_string()));
    }
    params
}

/// Today's date in UTC as (year, month, day).
pub(crate) fn today_utc() -> (i64, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_days(secs.div_euclid(86_400))
}

fn format_date((y, m, d): (i64, u32, u32)) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Proleptic Gregorian date for a count of days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[tauri::command]
pub async fn search_products(
    config: State<'_, Config>,
    query: String,
    limit: Option<u32>,
) -> Result<Value> {
    config.get("/products/search", &search_params(query, limit)).await
}

#[tauri::command]
pub async fn get_offers(config: State<'_, Config>, identifier: String) -> Result<Value> {
    config.get("/products/offers", &offers_params(identifier)).await
}

#[tauri::command]
pub async fn get_price_history(
    config: State<'_, Config>,
    identifier: String,
    days: Option<u32>,
) -> Result<Value> {
    config
        .get(
            "/products/offers/history",
            &price_history_params(identifier, days, today_utc()),
        )
        .await
}

#[tauri::command]
pub async fn get_deals(
    config: State<'_, Config>,
    options: Option<HashMap<String, Value>>,
) -> Result<Value> {
    config.get("/deals", &deals_params(options)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn get<'a>(params: &'a Params, key: &str) -> Vec<&'a str> {
        params
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    #[test]
    fn offers_use_the_ids_parameter() {
        assert_eq!(offers_params("B09XS7JWHH".into()), vec![("ids".to_string(), "B09XS7JWHH".to_string())]);
    }

    #[test]
    fn price_history_sends_ids_start_and_end() {
        let params = price_history_params("611247373064".into(), Some(180), (2026, 9, 27));
        assert_eq!(get(&params, "ids"), vec!["611247373064"]);
        assert_eq!(get(&params, "start"), vec!["2026-03-31"]);
        assert_eq!(get(&params, "end"), vec!["2026-09-27"]);
        assert!(get(&params, "id").is_empty() && get(&params, "days").is_empty());
    }

    #[test]
    fn price_history_defaults_to_30_days_and_crosses_year_boundaries() {
        let params = price_history_params("x".into(), None, (2026, 1, 10));
        assert_eq!(get(&params, "start"), vec!["2025-12-11"]);
        assert_eq!(get(&params, "end"), vec!["2026-01-10"]);
    }

    #[test]
    fn civil_date_round_trips() {
        for (y, m, d) in [(1970, 1, 1), (2000, 2, 29), (2024, 12, 31), (2026, 3, 1)] {
            assert_eq!(civil_from_days(days_from_civil(y, m, d)), (y, m, d));
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1) - days_from_civil(2000, 2, 28), 2);
    }

    #[test]
    fn deals_default_sort_is_one_the_api_accepts() {
        let params = deals_params(None);
        assert_eq!(get(&params, "sort"), vec!["hot"]);
        assert_eq!(get(&params, "limit"), vec!["20"]);
    }

    #[test]
    fn deals_options_override_defaults_without_duplicates_and_keep_all_value_types() {
        let mut options = HashMap::new();
        options.insert("limit".to_string(), json!(8));
        options.insert("sort".to_string(), json!("top-day"));
        options.insert("min_price".to_string(), json!(9.99));
        options.insert("is_bogo".to_string(), json!(true));
        options.insert("category".to_string(), json!("electronics"));
        options.insert("tag".to_string(), Value::Null);

        let params = deals_params(Some(options));
        assert_eq!(get(&params, "limit"), vec!["8"]);
        assert_eq!(get(&params, "sort"), vec!["top-day"]);
        assert_eq!(get(&params, "min_price"), vec!["9.99"]);
        assert_eq!(get(&params, "is_bogo"), vec!["true"]);
        assert_eq!(get(&params, "category"), vec!["electronics"]);
        assert!(get(&params, "tag").is_empty());
    }

    /// Serves exactly one HTTP response and returns the raw request it received.
    async fn one_shot_server(status_line: &'static str, body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let n = socket.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let response = format!(
                "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
            request
        });
        (format!("http://{addr}/v1"), handle)
    }

    fn config(base_url: String, api_key: &str) -> Config {
        let mut config = Config::new(api_key.to_string());
        config.base_url = base_url;
        config
    }

    #[tokio::test]
    async fn get_sends_auth_user_agent_and_query_and_parses_json() {
        let (base, server) = one_shot_server("HTTP/1.1 200 OK", r#"{"success":true,"data":[{"title":"Keurig K-Mini","offers":[]}]}"#).await;
        let value = config(base, "ss_live_0123456789abcdef0123456789abcdef")
            .get("/products/offers", &offers_params("611247373064".into()))
            .await
            .unwrap();
        let request = server.await.unwrap().to_lowercase();

        assert!(request.starts_with("get /v1/products/offers?ids=611247373064 http/1.1"), "{request}");
        assert!(request.contains("authorization: bearer ss_live_0123456789abcdef0123456789abcdef"));
        assert!(request.contains(&format!("user-agent: tauri-plugin-shopsavvy/{}", env!("CARGO_PKG_VERSION"))));
        assert_eq!(value["data"][0]["title"], "Keurig K-Mini");
    }

    #[tokio::test]
    async fn non_success_status_becomes_api_error_with_body() {
        let (base, server) = one_shot_server("HTTP/1.1 401 Unauthorized", r#"{"success":false,"error":"API key not found or has been revoked."}"#).await;
        let err = config(base, "ss_live_0123456789abcdef0123456789abcdef")
            .get("/deals", &deals_params(None))
            .await
            .unwrap_err();
        server.await.unwrap();

        match err {
            Error::Api(status, body) => {
                assert_eq!(status, 401);
                assert!(body.contains("revoked"));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn missing_api_key_fails_before_any_request() {
        let err = config("http://127.0.0.1:9/v1".into(), "  ")
            .get("/deals", &deals_params(None))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MissingApiKey));
    }
}
