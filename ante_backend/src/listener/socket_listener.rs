use axum::{
    body::Bytes,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures_util::SinkExt;
use futures_util::StreamExt;
use futures_util::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch::{
    Receiver as WatchReceiver, Sender as WatchSender, channel as watch_channel,
};
//TODO: make a dyn type
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseToWebSocket {
    response: String,
}
pub struct WebsocketMessageCommnand {
    message_type: WebSocketManagerCommandType,
    user_channel: Option<Sender<ResponseToWebSocket>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebSocketManagerCommandType {
    QuitWebsocket,
    QuitBountyID(i32),
    QuitFeed,
    ConnectFeed,
    ConnectBountyID(i32),
}
pub struct WebSocketManager {
    feed: HashMap<i32, Sender<ResponseToWebSocket>>,
    bounty_room: HashMap<i32, Sender<ResponseToWebSocket>>,
}
pub struct IDManager {
    id: i32,
}
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(id_manager): State<Arc<Mutex<IDManager>>>,
    State(cancel_chan): State<WatchReceiver<bool>>,
    State(websocket_manager_chan): State<Sender<WebsocketMessageCommnand>>,
) -> Response {
    ws.on_upgrade(|socket| {
        handle_websocket(socket, id_manager, cancel_chan, websocket_manager_chan)
    })
}
#[derive(Debug, Deserialize, Serialize)]
//TODO: maybe request more data in the future like ip ? for looging? maybe their wallet key?
pub struct UserContent {
    change_spot: WebSocketManagerCommandType,
}

pub async fn handle_websocket(
    mut socket: WebSocket,
    id_manager: Arc<Mutex<IDManager>>,
    cancel_chan: WatchReceiver<bool>,
    websocket_manager_chan: Sender<WebsocketMessageCommnand>,
) {
    let user_id = {
        let mut guard = id_manager.lock().await;
        let current_id = guard.id;
        guard.id += 1;
        current_id
    };
    let (mut sender, mut receiver) = socket.split();
    let (mut user_sender_Chan, mut user_reading): (
        Sender<ResponseToWebSocket>,
        Receiver<ResponseToWebSocket>,
    ) = tokio::sync::mpsc::channel(10);
    let cancel_chan_clone = cancel_chan.clone();
    //this is for reading chan
    let inner_websocket_manager_chan = websocket_manager_chan.clone();
    tokio::spawn(async move {
        //:TODO: replace with tokio::select with cancel_chan
        while let Some(msg) = receiver.next().await {
            if let Ok(valid_msg) = msg {
                if let Message::Text(text) = valid_msg {
                    let possible_content: Result<UserContent, serde_json::Error> =
                        serde_json::from_str(&text);
                    if let Ok(valid_content) = possible_content {
                        inner_websocket_manager_chan
                            .send(WebsocketMessageCommnand {
                                message_type: valid_content.change_spot,
                                user_channel: None,
                            })
                            .await;
                    }
                }
            } else {
                inner_websocket_manager_chan
                    .send(WebsocketMessageCommnand {
                        message_type: WebSocketManagerCommandType::QuitWebsocket,
                        user_channel: None,
                    })
                    .await;
            }
        }
    });
    //now send to the websocket_manager_chan the chan to which it can then write to
    websocket_manager_chan
        .send(WebsocketMessageCommnand {
            message_type: WebSocketManagerCommandType::ConnectFeed,
            user_channel: Some(user_sender_Chan),
        })
        .await;
    //write channel
    tokio::spawn(async move {
        //:TODO: add a tokio::select with cancel_chan
        while let Some(valid_message) = user_reading.recv().await {
            if let Ok(valid_json) = serde_json::to_vec(&valid_message) {
                sender
                    .send(Message::Binary(Bytes::copy_from_slice(&valid_json)))
                    .await;
            }
        }
    });
}
