// Checks the JS bindings against the Rust half of the plugin in this repo:
// every binding must invoke a command that the plugin registers, is allowed
// by the default permission set, and whose Rust parameter names match the
// argument keys sent from JS (Tauri maps camelCase JS keys to snake_case Rust
// params; all of ours are single words, so they must match exactly).
//
// Uses Tauri's own IPC mock (@tauri-apps/api/mocks) to capture invocations.

import { afterEach, describe, expect, test } from 'bun:test'
import { readFileSync } from 'fs'
import { join } from 'path'
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks'
import { getDeals, getOffers, getPriceHistory, searchProducts } from '../src/index'

const crateRoot = join(import.meta.dir, '..', '..')
const commandsRs = readFileSync(join(crateRoot, 'src', 'commands.rs'), 'utf8')
const libRs = readFileSync(join(crateRoot, 'src', 'lib.rs'), 'utf8')
const permissionsToml = readFileSync(join(crateRoot, 'permissions', 'default.toml'), 'utf8')

/** `#[tauri::command] pub async fn name(config: State<..>, a: T, b: Option<U>)` -> { name: ['a', 'b'] } */
function rustCommands(): Record<string, string[]> {
  const out: Record<string, string[]> = {}
  const re = /#\[tauri::command\]\s*pub\s+(?:async\s+)?fn\s+(\w+)\s*\(([\s\S]*?)\)\s*->/g
  for (const m of commandsRs.matchAll(re)) {
    const params = m[2]
      .split(/,(?![^<]*>)/)
      .map((p) => p.trim())
      .filter(Boolean)
      .map((p) => p.split(':')[0].trim())
      .filter((name) => name !== 'config' && name !== 'app' && name !== 'window')
    out[m[1]] = params
  }
  return out
}

const handlerBlock = libRs.match(/generate_handler!\[([\s\S]*?)\]/)?.[1] ?? ''
const registered = [...handlerBlock.matchAll(/commands::(\w+)/g)].map((m) => m[1])
const allowedByDefault = new Set(
  [...permissionsToml.matchAll(/commands\.allow\s*=\s*\[([^\]]*)\]/g)].flatMap((m) =>
    [...m[1].matchAll(/"(\w+)"/g)].map((x) => x[1]),
  ),
)

type Call = { cmd: string; args: Record<string, unknown> }
let calls: Call[] = []

function mockPlugin(responses: Record<string, unknown>) {
  calls = []
  mockIPC((cmd, args) => {
    calls.push({ cmd, args: (args ?? {}) as Record<string, unknown> })
    const command = cmd.replace('plugin:shopsavvy|', '')
    if (!(command in responses)) throw new Error(`unexpected command ${cmd}`)
    return responses[command]
  })
}

afterEach(() => clearMocks())

describe('guest-js bindings', () => {
  const searchResponse = { success: true, data: [{ shopsavvy: 'p1', title: 'AirPods Pro' }], pagination: { total: 1, limit: 10, offset: 0, returned: 1 } }
  const offersResponse = { success: true, data: [{ shopsavvy: 'p1', title: 'AirPods Pro', offers: [{ id: 'o1', retailer: 'amazon.com', price: 189.99 }] }] }
  const historyResponse = { success: true, data: [{ id: 'o1', retailer: 'amazon.com', history: [{ timestamp: '2026-09-01T00:00:00Z', price: 199 }] }] }
  const dealsResponse = { success: true, deals: [{ title: 'Deal', grade: { letter: 'A', value: 95 } }], pagination: { total: 1, has_more: false, limit: 8, offset: 0 } }

  test('each binding invokes the plugin command with the expected args and returns its result', async () => {
    mockPlugin({
      search_products: searchResponse,
      get_offers: offersResponse,
      get_price_history: historyResponse,
      get_deals: dealsResponse,
    })

    expect(await searchProducts('AirPods Pro', 10)).toEqual(searchResponse as never)
    expect(await getOffers('012345678905')).toEqual(offersResponse as never)
    expect(await getPriceHistory('012345678905', 180)).toEqual(historyResponse as never)
    expect(await getDeals({ category: 'electronics', limit: 8, sort: 'top-day' })).toEqual(dealsResponse as never)

    expect(calls).toEqual([
      { cmd: 'plugin:shopsavvy|search_products', args: { query: 'AirPods Pro', limit: 10 } },
      { cmd: 'plugin:shopsavvy|get_offers', args: { identifier: '012345678905' } },
      { cmd: 'plugin:shopsavvy|get_price_history', args: { identifier: '012345678905', days: 180 } },
      { cmd: 'plugin:shopsavvy|get_deals', args: { options: { category: 'electronics', limit: 8, sort: 'top-day' } } },
    ])
  })

  test('errors from the Rust command reject the promise', async () => {
    mockIPC(() => {
      throw 'ShopSavvy API error 401: invalid key'
    })
    await expect(getOffers('012345678905')).rejects.toBe('ShopSavvy API error 401: invalid key')
  })
})

describe('guest-js ↔ Rust contract', () => {
  const rust = rustCommands()

  test('the Rust side defines, registers, and allows by default exactly the commands the bindings call', async () => {
    mockPlugin({ search_products: {}, get_offers: {}, get_price_history: {}, get_deals: {} })
    await searchProducts('x')
    await getOffers('x')
    await getPriceHistory('x')
    await getDeals()
    const invoked = calls.map((c) => c.cmd.replace('plugin:shopsavvy|', '')).sort()

    expect(Object.keys(rust).sort()).toEqual(invoked)
    expect([...registered].sort()).toEqual(invoked)
    expect([...allowedByDefault].sort()).toEqual(invoked)
  })

  test('argument keys sent from JS match the Rust command parameter names', async () => {
    mockPlugin({ search_products: {}, get_offers: {}, get_price_history: {}, get_deals: {} })
    await searchProducts('x', 1)
    await getOffers('x')
    await getPriceHistory('x', 1)
    await getDeals({ limit: 1 })
    for (const { cmd, args } of calls) {
      const command = cmd.replace('plugin:shopsavvy|', '')
      expect({ command, params: Object.keys(args).sort() }).toEqual({ command, params: [...rust[command]].sort() })
    }
  })
})
