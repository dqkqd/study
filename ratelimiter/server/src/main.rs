use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let config = appconfig::Config::parse().unwrap();
    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind(config.server.addr())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> String {
    let now = chrono::Local::now()
        .timestamp_nanos_opt()
        .unwrap()
        .to_string();
    println!("Receive request at {now}");
    now
}
