# @shopsavvy/tauri-plugin

TypeScript bindings for [`tauri-plugin-shopsavvy`](https://github.com/shopsavvy/tauri-plugin-shopsavvy), a [Tauri](https://tauri.app/) v2 plugin for **product search, price comparison, price history, and deals** powered by the [ShopSavvy Data API](https://shopsavvy.com/data).

This package is the frontend half. The API calls run in the plugin's Rust half (the `tauri-plugin-shopsavvy` crate), so your API key never has to reach the webview.

[Documentation](https://shopsavvy.com/integrations/tauri) · [Get an API key](https://shopsavvy.com/data) · [Other integrations](https://shopsavvy.com/integrations)

## Install

Install both halves:

```bash
cargo add tauri-plugin-shopsavvy   # in src-tauri/
npm install @shopsavvy/tauri-plugin
```

Register the plugin in Rust and allow its commands (`"shopsavvy:default"` in your capability file). See the [plugin README](https://github.com/shopsavvy/tauri-plugin-shopsavvy#readme) for the Rust setup.

## Usage

```ts
import { searchProducts, getOffers, getPriceHistory, getDeals } from "@shopsavvy/tauri-plugin"

const results = await searchProducts("AirPods Pro", 10)
// results.data => [{ shopsavvy, title, brand, barcode, ... }]

const offers = await getOffers("012345678905")
// offers.data[0].offers => [{ retailer: "amazon.com", price: 189.99, URL, ... }]

const history = await getPriceHistory("012345678905", 180)
// history.data => [{ retailer, history: [{ timestamp, price }] }]

const deals = await getDeals({ category: "electronics", limit: 8, sort: "top-day" })
// deals.deals => [{ title, grade, pricing, retailer, url, votes }]
```

| Function | Arguments | Resolves to |
|----------|-----------|-------------|
| `searchProducts(query, limit?)` | keyword, optional max results | `SearchResponse` |
| `getOffers(identifier)` | barcode/UPC, ASIN, URL, model number, or ShopSavvy ID | `OffersResponse` |
| `getPriceHistory(identifier, days?)` | identifier, days of history | `PriceHistoryResponse` |
| `getDeals(options?)` | `sort` (`hot`, `new`, `top-hour`, `top-day`, `top-week`), `limit`, `offset`, `category`, `retailer`, `tag`, `grade`, `min_price`, `max_price` | `DealsResponse` |

Each function returns a promise that rejects with the error message from the Rust command if the request fails. All response types are exported.

## License

MIT
