import { invoke } from "@tauri-apps/api/core"

const PLUGIN = "plugin:shopsavvy"

export interface ProductOffer {
  retailer?: string
  price?: number
  condition?: string
  availability?: boolean
  url?: string
}

export interface ProductDetails {
  name?: string
  image?: string
  [key: string]: unknown
}

export interface PriceHistoryPoint {
  date?: string
  price?: number
}

export interface Deal {
  name?: string
  image?: string
  price?: number
  strikethrough?: number
  retailer?: string
  url?: string
}

interface ApiResponse<T> {
  data: T[]
}

export interface DealsOptions {
  category?: string
  limit?: number
  sort?: "trending" | "price" | "discount"
  grade?: string
}

export async function searchProducts(query: string, limit?: number): Promise<ApiResponse<ProductDetails>> {
  return invoke(`${PLUGIN}|search_products`, { query, limit })
}

export async function getOffers(identifier: string): Promise<ApiResponse<ProductOffer>> {
  return invoke(`${PLUGIN}|get_offers`, { identifier })
}

export async function getPriceHistory(identifier: string, days?: number): Promise<ApiResponse<PriceHistoryPoint>> {
  return invoke(`${PLUGIN}|get_price_history`, { identifier, days })
}

export async function getDeals(options?: DealsOptions): Promise<ApiResponse<Deal>> {
  return invoke(`${PLUGIN}|get_deals`, { options })
}
