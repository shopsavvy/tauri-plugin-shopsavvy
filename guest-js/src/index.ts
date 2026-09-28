import { invoke } from "@tauri-apps/api/core"

const PLUGIN = "plugin:shopsavvy"

// The Rust commands return the ShopSavvy Data API's JSON response unchanged,
// so these types describe the Data API payloads
// (https://shopsavvy.com/data/documentation).

export interface ApiMeta {
  credits_used: number
  credits_remaining: number
  rate_limit_remaining?: number
}

// The API passes these product/offer fields straight through from their records, so
// an unknown value arrives as an explicit JSON `null`, not an absent key.
export interface Product {
  /** ShopSavvy product ID */
  shopsavvy: string
  title: string
  brand?: string | null
  category?: string | null
  images?: string[]
  barcode?: string | null
  /** Amazon ASIN */
  amazon?: string | null
  model?: string | null
  mpn?: string | null
  color?: string | null
  [key: string]: unknown
}

export interface Offer {
  id: string
  /** Retailer name, e.g. "Amazon" */
  retailer?: string | null
  price?: number | null
  currency?: string | null
  /** "in" / "out"; absent when unknown. */
  availability?: string
  condition?: string | null
  URL?: string | null
  /** Marketplace seller; null on first-party offers. */
  seller?: string | null
  timestamp?: string | null
  [key: string]: unknown
}

export interface ProductWithOffers extends Product {
  offers: Offer[]
}

export interface PriceHistoryPoint {
  timestamp: string
  price: number
  /** Null on an archived point with no recorded currency — never assume USD. */
  currency?: string | null
  /** "in" / "out"; absent when unknown. */
  availability?: string
}

export interface OfferWithHistory extends Offer {
  /** Newest first. Empty when the window has no points (e.g. eBay listings). */
  history: PriceHistoryPoint[]
}

/** One product in a price-history response: the product, its offers, each offer's history. */
export interface ProductWithOfferHistory extends Product {
  offers: OfferWithHistory[]
}

export interface SearchResponse {
  success: boolean
  data: Product[]
  pagination: { total: number; limit: number; offset: number; returned: number }
  meta?: ApiMeta
}

export interface OffersResponse {
  success: boolean
  data: ProductWithOffers[]
  meta?: ApiMeta
}

/**
 * GET /products/offers/history, passed through unchanged by the Rust command:
 * one entry PER PRODUCT, each with its offers, each offer carrying its history.
 */
export interface PriceHistoryResponse {
  success: boolean
  data: ProductWithOfferHistory[]
  meta?: ApiMeta
}

export interface Deal {
  path: string
  title: string
  subtitle?: string
  description?: string
  grade: { letter: string; suffix?: string; value: number; justification?: string }
  pricing: { current: number; original?: number; currency: string }
  retailer: { name: string }
  product?: string
  url: string
  image?: { url: string }
  votes: { upvotes: number; downvotes: number; score: number }
  comment_count: number
  tags?: { slug: string; display: string }[]
  expires_at?: string
  created_at: string
}

export interface DealsResponse {
  success: boolean
  deals: Deal[]
  pagination: { total: number; has_more: boolean; limit: number; offset: number }
  meta?: ApiMeta
}

export interface DealsOptions {
  sort?: "hot" | "new" | "top-hour" | "top-day" | "top-week"
  limit?: number
  offset?: number
  category?: string
  retailer?: string
  tag?: string
  grade?: string
  min_price?: number
  max_price?: number
}

/** Search products by keyword. */
export async function searchProducts(query: string, limit?: number): Promise<SearchResponse> {
  return invoke<SearchResponse>(`${PLUGIN}|search_products`, { query, limit })
}

/**
 * Current offers across retailers. `identifier` can be a barcode/UPC, ASIN,
 * product URL, model number, or ShopSavvy product ID.
 */
export async function getOffers(identifier: string): Promise<OffersResponse> {
  return invoke<OffersResponse>(`${PLUGIN}|get_offers`, { identifier })
}

/** Price history for the last `days` days: products -> offers -> history (newest first). */
export async function getPriceHistory(identifier: string, days?: number): Promise<PriceHistoryResponse> {
  return invoke<PriceHistoryResponse>(`${PLUGIN}|get_price_history`, { identifier, days })
}

/** Current deals with expert grades and community votes. */
export async function getDeals(options?: DealsOptions): Promise<DealsResponse> {
  return invoke<DealsResponse>(`${PLUGIN}|get_deals`, { options })
}
