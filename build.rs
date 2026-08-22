fn main() {
    // Automatically configure features based on target
    let target = std::env::var("TARGET").unwrap_or_else(|_| String::new());

    if target.contains("wasm32") {
        // Set the getrandom backend configuration
        println!("cargo:rustc-cfg=getrandom_backend=\"wasm_js\"");
    }

    // This ensures that the build script is re-run when the target changes
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-changed=build.rs");
}
