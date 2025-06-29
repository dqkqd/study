use std::thread;

#[tokio::main]
async fn main() {
    let config = appconfig::Config::parse().unwrap();
    let server_url = format!("http://{}", config.ratelimiter.addr());
    let freq = config.client.frequency();
    loop {
        run(&server_url).await;
        thread::sleep(freq);
    }
}

async fn run(url: &str) {
    match reqwest::get(url).await.unwrap().error_for_status() {
        Ok(r) => println!("Server response {}", r.text().await.unwrap()),
        Err(e) => println!("{}", e),
    }
}
