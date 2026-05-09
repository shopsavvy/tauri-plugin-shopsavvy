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
const deals   = await getDeals({ category: "electronics", limit: 8, sort: "trending" })
```

## Test

```bash
./test.sh
```

## License

MIT — see [LICENSE](./LICENSE).
