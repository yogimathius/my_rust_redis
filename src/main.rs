use clap::Parser;
use redis_starter_rust::{app_state::AppState, log};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    port: Option<u16>,
    #[clap(short, long)]
    replicaof: Option<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let args = Args::parse();
    let port = args.port.unwrap_or(6379);

    let app_state = AppState::new(port, args.replicaof);

    // Start the server setup task
    let server_clone = Arc::clone(&app_state.server);
    let notify_clone = Arc::clone(&app_state.notify);
    tokio::spawn(async move {
        let mut server = server_clone.lock().await;
        server.run().await;
        notify_clone.notify_one();
    });

    // Wait for the server to complete its setup
    app_state.notify.notified().await;
    log!("Server setup complete");

    // Start workers
    let handles = app_state.start_workers();

    // Wait for all worker tasks to finish
    for handle in handles {
        handle.await.expect("Failed to join worker task");
    }
}
