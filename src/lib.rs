//! Tauri v2 plugin for the ShopSavvy Data API.
//!
//! ```ignore
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_shopsavvy::init())
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```

mod commands;
mod error;

pub use error::{Error, Result};

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

const PLUGIN_NAME: &str = "shopsavvy";

pub struct Builder {
    api_key: Option<String>,
    base_url: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
        }
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let api_key = self
            .api_key
            .or_else(|| std::env::var("SHOPSAVVY_API_KEY").ok())
            .unwrap_or_default();

        PluginBuilder::new(PLUGIN_NAME)
            .invoke_handler(tauri::generate_handler![
                commands::search_products,
                commands::get_offers,
                commands::get_price_history,
                commands::get_deals,
            ])
            .setup(move |app, _api| {
                let mut config = commands::Config::new(api_key.clone());
                if let Some(url) = self.base_url.clone() {
                    config.base_url = url;
                }
                app.manage(config);
                Ok(())
            })
            .build()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience helper for the default configuration (reads `SHOPSAVVY_API_KEY` from env).
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new().build()
}
