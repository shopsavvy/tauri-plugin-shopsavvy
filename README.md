# tauri-plugin-shopsavvy

[Tauri](https://tauri.app/) v2 plugin for **product search, price comparison, price history, and deals** powered by the [ShopSavvy Data API](https://shopsavvy.com/data). Ships a Rust crate (the plugin runtime) and a TypeScript bindings package (the renderer-side wrapper).

[Documentation](https://shopsavvy.com/integrations/tauri) · [Get an API key](https://shopsavvy.com/data) · [Other integrations](https://shopsavvy.com/integrations)

## Install

Install both halves:

```bash
cargo add tauri-plugin-shopsavvy
npm install @shopsavvy/tauri-plugin
```

## Wire the plugin in Rust

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shopsavvy::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`init()` reads `SHOPSAVVY_API_KEY` from the environment. To override:

```rust
.plugin(
    tauri_plugin_shopsavvy::Builder::new()
        .api_key("ss_live_…")
        .build(),
)
```

## Allow the commands

In `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "shopsavvy:default"
  ]
}
```

## Use from the frontend

```ts
import { searchProducts, getOffers, getPriceHistory, getDeals } from "@shopsavvy/tauri-plugin"

const results = await searchProducts("AirPods Pro", 10)
const offers  = await getOffers("012345678905")
const history = await getPriceHistory("012345678905", 180)
const deals   = await getDeals({ category: "electronics", limit: 8, sort: "top-day" })
```

Every command returns the Data API's JSON response unchanged ([response formats](https://shopsavvy.com/data/documentation)). Failures reject with a message string, e.g. `API returned status 401: {...}` or the missing-API-key error.

| Command | Data API request |
|---|---|
| `search_products(query, limit?)` | `GET /v1/products/search?q=&limit=` |
| `get_offers(identifier)` | `GET /v1/products/offers?ids=` (barcode, ASIN, URL, model number or ShopSavvy ID) |
| `get_price_history(identifier, days?)` | `GET /v1/products/offers/history?ids=&start=&end=`, covering the last `days` days (default 30, UTC) |
| `get_deals(options?)` | `GET /v1/deals`, options passed through as query parameters; `sort` defaults to `hot`, `limit` to 20 |

## Test

```bash
./test.sh
```

Runs the Rust unit tests, the plugin IPC tests (the plugin mounted on Tauri's mock runtime with `shopsavvy:default` granted, answering from a local stand-in for the API), and the guest-js checks.

## License

MIT — see [LICENSE](./LICENSE).
