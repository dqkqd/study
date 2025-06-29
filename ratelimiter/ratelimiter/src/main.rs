use anyhow::Result;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use clap::{Parser, ValueEnum};
use ratelimiter::{Ratelimit, Ratelimiter};
use serde::Serialize;

#[derive(Parser)]
struct Args {
    algo: Algo,
}

#[derive(Clone, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Algo {
    TokenBucket,
    FixedWindowCounter,
    SlidingWindowLog,
}

impl Algo {
    fn ratelimiter(&self, config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        match self {
            Algo::TokenBucket => Ratelimiter::token_bucket(config),
            Algo::FixedWindowCounter => Ratelimiter::fixed_window_counter(config),
            Algo::SlidingWindowLog => Ratelimiter::sliding_window_log(config),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub ratelimiter: Ratelimiter,
    pub server_url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = appconfig::Config::parse().unwrap();
    let ratelimiter = args.algo.ratelimiter(&config.ratelimiter);
    let server_url = format!("http://{}", config.server.addr());
    let state = AppState {
        ratelimiter,
        server_url,
    };
    let router = Router::new().route("/", get(throttle)).with_state(state);
    let listener = tokio::net::TcpListener::bind(config.ratelimiter.addr())
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn throttle(state: State<AppState>) -> Result<String, StatusCode> {
    if state.ratelimiter.try_accept().is_ok() {
        let r = reqwest::get(&state.server_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        Ok(r)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
