// `sqlx::migrate!()` embeds the migration files at compile time. This makes
// Cargo rebuild the app whenever a migration is added or changed.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
