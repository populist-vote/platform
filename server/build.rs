fn main() {
    // Stable Rust does not automatically track directory changes inside migrate!.
    // A migration-only commit must rebuild the binary's embedded history.
    println!("cargo:rerun-if-changed=../db/migrations");
}
