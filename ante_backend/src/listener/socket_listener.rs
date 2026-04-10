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
pub enum ResponseToWebSocketCommandType {
    RecentWinner,
    RecentVote,
    RecentPost,
    RecentAnswer,
}
pub trait ResponseToWebSocket: Send {
    fn get_response_type(&self) -> ResponseToWebSocketCommandType;
    fn serialize(&self) -> Bytes;
}
pub struct WebsocketMessageCommnand {
    message_type: Option<WebSocketManagerCommandType>,
    user_channel: Option<Sender<Box<dyn ResponseToWebSocket>>>,
    user_id: Option<i32>,
    block_chain_event: Option<BlockchainEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebSocketManagerCommandType {
    QuitWebsocket,
    QuitBountyID(i32),
    QuitFeed,
    ConnectFeed,
    ConnectBountyID(i32),
}
#[derive(Debug, Deserialize, Serialize)]
pub enum BlockchainEvent {
    NewWinner,
    NewVote,
    NewPost,
    NewAnswer,
}
pub struct WebSocketManager {
    feed: HashMap<i32, Sender<Box<dyn ResponseToWebSocket>>>,
    bounty_room: HashMap<i32, Sender<Box<dyn ResponseToWebSocket>>>,
}
impl WebSocketManager {
    pub fn init() -> Self {
        WebSocketManager {
            feed: HashMap::new(),
            bounty_room: HashMap::new(),
        }
    }
    pub async fn handle_websocket_messages(
        &mut self,
        mut reading_chan: Receiver<WebsocketMessageCommnand>,
        cancel_chan: WatchReceiver<bool>,
    )
    //TODO: add a tokio::select with cancel_chan
    {
        while let Some(command) = reading_chan.recv().await {
            if command.user_id.is_some() && command.message_type.is_some() {
                let user_id = command.user_id.unwrap();
                match command.message_type.unwrap() {
                    WebSocketManagerCommandType::QuitWebsocket => {
                        self.feed.remove(&user_id);
                        self.bounty_room.remove(&user_id);
                    }
                    WebSocketManagerCommandType::QuitBountyID(bounty_id) => {
                        self.bounty_room.remove(&bounty_id);
                    }
                    WebSocketManagerCommandType::QuitFeed => {
                        self.feed.remove(&user_id);
                    }
                    WebSocketManagerCommandType::ConnectFeed => {
                        if let Some(user_channel) = command.user_channel {
                            self.feed.insert(user_id, user_channel);
                        }
                    }
                    WebSocketManagerCommandType::ConnectBountyID(bounty_id) => {
                        if let Some(user_channel) = command.user_channel {
                            self.bounty_room.insert(bounty_id, user_channel);
                        }
                    }
                }
                continue;
            }
            if command.block_chain_event.is_some() {
                let event = command.block_chain_event.unwrap();
                //TODO: eventually build out custom message for each type after parsing is done!
                match event {
                    BlockchainEvent::NewWinner => for (_, channel) in self.feed.iter() {},
                    BlockchainEvent::NewVote => for (_, channel) in self.feed.iter() {},
                    BlockchainEvent::NewPost => for (_, channel) in self.feed.iter() {},
                    BlockchainEvent::NewAnswer => {}
                }
            }
        }
    }
}
pub struct IDManager {
    id: i32,
}
impl IDManager {
    pub fn init() -> Self {
        IDManager { id: 0 }
    }
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
        Sender<Box<dyn ResponseToWebSocket>>,
        Receiver<Box<dyn ResponseToWebSocket>>,
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
                                message_type: Some(valid_content.change_spot),
                                user_channel: None,
                                user_id: Some(user_id),
                                block_chain_event: None,
                            })
                            .await;
                    }
                }
            } else {
                inner_websocket_manager_chan
                    .send(WebsocketMessageCommnand {
                        message_type: Some(WebSocketManagerCommandType::QuitWebsocket),
                        user_channel: None,
                        user_id: Some(user_id),
                        block_chain_event: None,
                    })
                    .await;
            }
        }
    });
    //now send to the websocket_manager_chan the chan to which it can then write to
    websocket_manager_chan
        .send(WebsocketMessageCommnand {
            message_type: Some(WebSocketManagerCommandType::ConnectFeed),
            user_channel: Some(user_sender_Chan),
            user_id: Some(user_id),
            block_chain_event: None,
        })
        .await;
    //write channel
    tokio::spawn(async move {
        //:TODO: add a tokio::select with cancel_chan
        while let Some(valid_message) = user_reading.recv().await {
            let valid_json = valid_message.serialize();
            sender.send(Message::Binary(valid_json)).await;
        }
    });
}
