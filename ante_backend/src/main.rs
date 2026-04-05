use axum::{Router, routing::get};
use std::env::var;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch::{
    Receiver as WatchReceiver, Sender as WatchSender, channel as watch_channel,
};
use tokio::task::JoinHandle;

use crate::listener::anchor_listener::listen_to_program;
mod db_data;
mod listener;
#[tokio::main]
async fn main() {
    let mut handlers: Vec<JoinHandle<()>> = Vec::new();
    dotenvy::dotenv().ok();
    let db_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("{}", db_url);
    let db = db_data::postgres_runner::create_pool(&db_url).await;

    let (tx_event_sender, mut tx_event_receiver): (
        Sender<listener::anchor_listener::ProgramEvent>,
        Receiver<listener::anchor_listener::ProgramEvent>,
    ) = tokio::sync::mpsc::channel(100);

    let (tx, rx) = watch_channel(false);

    let rx_clone = rx.clone();
    let handle = tokio::spawn(async move {
        listen_to_program(tx_event_sender, rx_clone).await;
    });
    handlers.push(handle);

    let mut count = 0;
    loop {
        let new_content = tx_event_receiver.recv().await;
        count += 1;
        if count == 5 {
            tx.send(true);
        }
    }
    return;
    ///acknowledge that this won't even run!
    println!("server is running");
    let app: Router = Router::new().route("/", get(|| async { "server is up" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
