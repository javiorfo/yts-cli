mod app;
mod config;
mod downloads;
mod elements;

#[tokio::main]
async fn main() {
    if let Err(e) = app::run().await {
        eprintln!("{e}");
    }
}
