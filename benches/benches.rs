#![allow(missing_docs)]

extern crate rust_simple_chat as app;

use std::{
    future::IntoFuture,
    io::BufRead,
    process::{Command, Stdio},
    sync::Arc,
};

use app::{api::State, infra::repositories};
use axum::{Extension, Router, routing::get};
use tower::ServiceBuilder;

use crate::app::api;

fn main() {
    if on_ci() {
        install_rewrk();
    } else {
        ensure_rewrk_is_installed();
    }

    let handler = State {
        messages_repository: Arc::new(
            repositories::messages::MockMessagesRepositoryTrait::default(),
        ),
    };

    benchmark("login").path("/login").method("post").run(|| {
        Router::new()
            .route("/login", get(api::v1::login::login_handler))
            .layer(ServiceBuilder::new().layer(Extension(handler)))
    });
}

fn benchmark(name: &'static str) -> BenchmarkBuilder {
    BenchmarkBuilder {
        name,
        path: None,
        method: None,
        headers: None,
        body: None,
    }
}

struct BenchmarkBuilder {
    name: &'static str,
    path: Option<&'static str>,
    method: Option<&'static str>,
    headers: Option<&'static [(&'static str, &'static str)]>,
    body: Option<&'static str>,
}

macro_rules! config_method {
    ($name:ident, $ty:ty) => {
        fn $name(mut self, $name: $ty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

impl BenchmarkBuilder {
    config_method!(path, &'static str);
    config_method!(method, &'static str);
    config_method!(headers, &'static [(&'static str, &'static str)]);
    config_method!(body, &'static str);

    fn run<F>(self, f: F)
    where
        F: FnOnce() -> Router<()>,
    {
        // support only running some benchmarks with
        // ```
        // cargo bench -- routing send-json
        // ```
        let args = std::env::args().collect::<Vec<_>>();
        if args.len() != 1 {
            let names = &args[1..args.len() - 1];
            if !names.is_empty() && !names.contains(&self.name.to_owned()) {
                return;
            }
        }

        let app = f();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let listener = rt
            .block_on(tokio::net::TcpListener::bind("0.0.0.0:0"))
            .unwrap();
        let addr = listener.local_addr().unwrap();

        std::thread::spawn(move || {
            rt.block_on(axum::serve(listener, app).into_future())
                .unwrap();
        });

        let mut cmd = Command::new("rewrk");
        cmd.stdout(Stdio::piped());

        cmd.arg("--host");
        cmd.arg(format!("http://{addr}{}", self.path.unwrap_or("")));

        cmd.args(["--connections", "10"]);
        cmd.args(["--threads", "10"]);

        if on_ci() {
            // don't slow down CI by running the benchmarks for too long
            // but do run them for a bit
            cmd.args(["--duration", "1s"]);
        } else {
            cmd.args(["--duration", "10s"]);
        }

        if let Some(method) = self.method {
            cmd.args(["--method", method]);
        }

        for (key, value) in self.headers.into_iter().flatten() {
            cmd.arg("--header");
            cmd.arg(format!("{key}: {value}"));
        }

        if let Some(body) = self.body {
            cmd.args(["--body", body]);
        }

        eprintln!("Running {:?} benchmark", self.name);

        // indent output from `rewrk` so it's easier to read when running multiple benchmarks
        let mut child = cmd.spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdout = std::io::BufReader::new(stdout);
        for line in stdout.lines() {
            let line = line.unwrap();
            println!("  {line}");
        }

        let status = child.wait().unwrap();

        if !status.success() {
            eprintln!("`rewrk` command failed");
            std::process::exit(status.code().unwrap());
        }
    }
}

fn install_rewrk() {
    println!("installing rewrk");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "install",
        "rewrk",
        "--git",
        "https://github.com/ChillFish8/rewrk.git",
    ]);
    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("failed to install rewrk"));
    if !status.success() {
        panic!("failed to install rewrk");
    }
}

fn ensure_rewrk_is_installed() {
    let mut cmd = Command::new("rewrk");
    cmd.arg("--help");
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.status().unwrap_or_else(|_| {
        panic!("rewrk is not installed. See https://github.com/lnx-search/rewrk")
    });
}

fn on_ci() -> bool {
    std::env::var("GITHUB_ACTIONS").is_ok()
}
