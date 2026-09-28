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

export interface Product {
  /** ShopSavvy product ID */
  shopsavvy: string
  title: string
  brand?: string
  category?: string
  images?: string[]
  barcode?: string
  /** Amazon ASIN */
  amazon?: string
  model?: string
  mpn?: string
  color?: string
  [key: string]: unknown
}

export interface Offer {
  id: string
  /** Retailer domain, e.g. "amazon.com" */
  retailer?: string
  price?: number
  currency?: string
  availability?: string
  condition?: string
  URL?: string
  seller?: string
  timestamp?: string
  [key: string]: unknown
}

export interface ProductWithOffers extends Product {
  offers: Offer[]
}

export interface PriceHistoryPoint {
  timestamp: string
  price: number
  currency?: string | null
  availability?: string
}

export interface OfferWithHistory extends Offer {
  history: PriceHistoryPoint[]
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

export interface PriceHistoryResponse {
  success: boolean
  data: OfferWithHistory[]
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

/** Price history for the last `days` days, per retailer offer. */
export async function getPriceHistory(identifier: string, days?: number): Promise<PriceHistoryResponse> {
  return invoke<PriceHistoryResponse>(`${PLUGIN}|get_price_history`, { identifier, days })
}

/** Current deals with expert grades and community votes. */
export async function getDeals(options?: DealsOptions): Promise<DealsResponse> {
  return invoke<DealsResponse>(`${PLUGIN}|get_deals`, { options })
}
