use axum::{
    Router,
    extract::FromRef,
    routing::{any, get},
};
use futures_util::lock::Mutex;
use std::env::var;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use tokio::sync::watch::{
    Receiver as WatchReceiver, Sender as WatchSender, channel as watch_channel,
};
use tokio::task::JoinHandle;

use crate::{
    db_data::postgres_runner::{DbCommand, run_db_requests},
    listener::socket_listener::{BlockchainEvent, IDManager, WebsocketMessageCommnand, ws_handler},
};
use crate::{
    decryption::logs_reader::read_program_logs, listener::anchor_listener::listen_to_program,
};

#[derive(Clone, FromRef, Debug)]
pub struct AppState {
    pub id_manager: Arc<Mutex<IDManager>>,
    pub cancel_chan: WatchReceiver<bool>,
    pub websocket_manager_chan: Sender<WebsocketMessageCommnand>,
}
mod db_data;
mod decryption;
mod listener;
#[tokio::main]
async fn main() {
    let mut handlers: Vec<JoinHandle<()>> = Vec::new();
    dotenvy::dotenv().ok();
    let db_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = db_data::postgres_runner::create_pool(&db_url).await;
    let (db_request_sender, db_request_receiver): (Sender<DbCommand>, Receiver<DbCommand>) =
        tokio::sync::mpsc::channel(100);

    let db_handler = tokio::spawn(async move {
        run_db_requests(db_request_receiver, &db).await;
    });
    handlers.push(db_handler);
    let (tx_event_sender, tx_event_receiver): (
        Sender<listener::anchor_listener::ProgramEvent>,
        Receiver<listener::anchor_listener::ProgramEvent>,
    ) = tokio::sync::mpsc::channel(100);

    let (cancellation_sender, cancellation_listener) = watch_channel(false);

    let rx_clone = cancellation_listener.clone();
    let handle = tokio::spawn(async move {
        listen_to_program(tx_event_sender, rx_clone).await;
    });
    handlers.push(handle);
    let mut websocket_manager = listener::socket_listener::WebSocketManager::init();
    let (websocket_manager_chan_sender, websocket_manager_chan_receiver): (
        Sender<WebsocketMessageCommnand>,
        Receiver<WebsocketMessageCommnand>,
    ) = tokio::sync::mpsc::channel(10);

    let websocket_sender_clone = websocket_manager_chan_sender.clone();
    tokio::spawn(async move {
        read_program_logs(tx_event_receiver, websocket_sender_clone, db_request_sender).await;
    });
    let rx_clone = cancellation_listener.clone();
    tokio::spawn(async move {
        websocket_manager
            .handle_websocket_messages(websocket_manager_chan_receiver, rx_clone)
            .await;
    });

    let id_manager = IDManager::init();
    let app_state = AppState {
        id_manager: Arc::new(Mutex::new(id_manager)),
        cancel_chan: cancellation_listener.clone(),
        websocket_manager_chan: websocket_manager_chan_sender.clone(),
    };
    let app: Router = Router::new()
        .route("/", get(|| async { "server is up" }))
        .route("/websocket", any(ws_handler))
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
