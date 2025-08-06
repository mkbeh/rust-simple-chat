fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=VERSION={version}");
}
