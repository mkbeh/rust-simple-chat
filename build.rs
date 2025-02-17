use std::process::Command;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let git_version = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap_or_else(|_| String::from("UNKNOWN"));

    println!("cargo:rustc-env=GIT_COMMIT_ID={git_version}");

    println!(
        "cargo:rustc-env=APP_VERSION={}({})",
        version,
        git_version.trim()
    );
}
