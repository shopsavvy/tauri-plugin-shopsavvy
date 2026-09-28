// @tauri-apps/api/mocks installs its IPC mock on `window`, so the tests need a DOM.
import { GlobalRegistrator } from '@happy-dom/global-registrator'

GlobalRegistrator.register()
