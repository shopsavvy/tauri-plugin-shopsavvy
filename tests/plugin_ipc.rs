//! Drives the plugin the way a Tauri app does: registered on a (mock) app,
//! commands invoked over IPC as `plugin:shopsavvy|<command>` with the argument
//! names the JS bindings send, answered by a local stand-in for the Data API.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::WebviewWindowBuilder;
use tauri_utils::acl::capability::Capability;
use tauri_utils::acl::manifest::{Manifest, PermissionFile};
use tauri_utils::acl::resolved::Resolved;
use tauri_utils::platform::Target;

/// A context whose ACL is resolved exactly as tauri-build does for an app:
/// this crate's permissions/default.toml as the `shopsavvy` plugin manifest,
/// and one capability granting "shopsavvy:default" to the "main" window. So
/// these tests also prove the default permission set allows every command.
fn context_granting_shopsavvy_default() -> tauri::Context<tauri::test::MockRuntime> {
    let permissions: PermissionFile =
        toml::from_str(include_str!("../permissions/default.toml")).expect("permissions/default.toml parses");
    let acl = BTreeMap::from([("shopsavvy".to_string(), Manifest::new(vec![permissions], None))]);
    let capability: Capability = serde_json::from_value(json!({
        "identifier": "main-capability",
        "windows": ["main"],
        "permissions": ["shopsavvy:default"]
    }))
    .unwrap();
    let resolved = Resolved::resolve(
        &acl,
        BTreeMap::from([("main-capability".to_string(), capability)]),
        Target::current(),
    )
    .expect("shopsavvy:default resolves");

    let mut context = mock_context(noop_assets());
    *context.runtime_authority_mut() = tauri::ipc::RuntimeAuthority::new(acl, resolved);
    context
}

/// Answers one HTTP request with `body` and reports the request line it received.
fn one_shot_api(status_line: &'static str, body: &'static str) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buf = vec![0u8; 8192];
        let n = socket.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        let response = format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).unwrap();
        tx.send(request.lines().next().unwrap_or_default().to_string()).unwrap();
    });
    (format!("http://{addr}/v1"), rx)
}

fn invoke(base_url: String, command: &str, args: Value) -> Result<Value, Value> {
    let app = mock_builder()
        .plugin(
            tauri_plugin_shopsavvy::Builder::new()
                .api_key("ss_live_0123456789abcdef0123456789abcdef")
                .base_url(base_url)
                .build(),
        )
        .build(context_granting_shopsavvy_default())
        .expect("failed to build mock app");
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("failed to build webview");

    get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: format!("plugin:shopsavvy|{command}"),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|body| body.deserialize::<Value>().unwrap())
}

#[test]
fn get_offers_over_ipc_hits_products_offers_with_ids() {
    let (base, request) = one_shot_api(
        "HTTP/1.1 200 OK",
        r#"{"success":true,"data":[{"title":"Keurig K-Mini","offers":[{"id":"o1","retailer":"Amazon","price":58.86}]}]}"#,
    );

    let response = invoke(base, "get_offers", json!({ "identifier": "611247373064" })).unwrap();

    assert_eq!(request.recv().unwrap(), "GET /v1/products/offers?ids=611247373064 HTTP/1.1");
    assert_eq!(response["data"][0]["offers"][0]["price"], 58.86);
}

#[test]
fn get_price_history_over_ipc_sends_start_and_end() {
    let (base, request) = one_shot_api("HTTP/1.1 200 OK", r#"{"success":true,"data":[]}"#);

    invoke(base, "get_price_history", json!({ "identifier": "611247373064", "days": 7 })).unwrap();

    let line = request.recv().unwrap();
    assert!(line.starts_with("GET /v1/products/offers/history?ids=611247373064&start="), "{line}");
    assert!(line.contains("&end="), "{line}");
}

#[test]
fn search_and_deals_over_ipc() {
    let (base, request) = one_shot_api("HTTP/1.1 200 OK", r#"{"success":true,"data":[],"pagination":{"total":0}}"#);
    invoke(base, "search_products", json!({ "query": "airpods pro", "limit": 5 })).unwrap();
    assert_eq!(request.recv().unwrap(), "GET /v1/products/search?q=airpods+pro&limit=5 HTTP/1.1");

    let (base, request) = one_shot_api("HTTP/1.1 200 OK", r#"{"success":true,"deals":[]}"#);
    invoke(base, "get_deals", json!({ "options": { "category": "electronics", "limit": 8, "sort": "top-day" } })).unwrap();
    assert_eq!(request.recv().unwrap(), "GET /v1/deals?category=electronics&limit=8&sort=top-day HTTP/1.1");
}

#[test]
fn api_errors_reach_the_frontend_as_messages() {
    let (base, _request) = one_shot_api(
        "HTTP/1.1 401 Unauthorized",
        r#"{"success":false,"error":"API key not found or has been revoked."}"#,
    );

    let err = invoke(base, "get_deals", json!({})).unwrap_err();

    let message = err.as_str().unwrap_or_default();
    assert!(message.starts_with("API returned status 401"), "{err}");
    assert!(message.contains("revoked"), "{err}");
}
