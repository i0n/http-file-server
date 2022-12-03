fn main() {
    // If we set CARGO_PKG_VERSION this way, then it will override the default value, which is
    // taken from the `version` in Cargo.toml.
    if let Ok(val) = std::env::var("VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=VERSION");
    // Set other build flags...
    if let Ok(val) = std::env::var("REV") {
        println!("cargo:rustc-env=REV={}", val);
    } else {
        println!("cargo:rustc-env=REV={}", "");
    }
    println!("cargo:rerun-if-env-changed=REV");
    if let Ok(val) = std::env::var("BRANCH") {
        println!("cargo:rustc-env=BRANCH={}", val);
    } else {
        println!("cargo:rustc-env=BRANCH={}", "");
    }
    println!("cargo:rerun-if-env-changed=BRANCH");
    if let Ok(val) = std::env::var("BUILD_USER") {
        println!("cargo:rustc-env=BUILD_USER={}", val);
    } else {
        println!("cargo:rustc-env=BUILD_USER={}", "");
    }
    println!("cargo:rerun-if-env-changed=BUILD_USER");
    if let Ok(val) = std::env::var("RUST_VERSION") {
        println!("cargo:rustc-env=RUST_VERSION={}", val);
    } else {
        println!("cargo:rustc-env=RUST_VERSION={}", "");
    }
    println!("cargo:rerun-if-env-changed=RUST_VERSION");
}
