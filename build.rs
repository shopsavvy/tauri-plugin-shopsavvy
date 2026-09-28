// Tauri v2 resolves plugin permissions at app build time: this build script
// exports permissions/*.toml (via the `links` key in Cargo.toml) so an app's
// capability can grant "shopsavvy:default". Without it the app build fails
// with "Permission shopsavvy:default not found".
//
// The command permissions (allow-search-products, ...) are written by hand in
// permissions/default.toml, so no command list is passed here: autogenerating
// them as well would define each identifier twice.
fn main() {
    tauri_plugin::Builder::new(&[]).build();
}
