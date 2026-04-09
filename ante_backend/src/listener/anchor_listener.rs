use solana_client::rpc_config::*;
use solana_client::{nonblocking::pubsub_client::PubsubClient, rpc_response::UiTransactionError};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch::{
    Receiver as WatchReceiver, Sender as WatchSender, channel as watch_channel,
};
use tokio_stream::StreamExt;
pub enum ActionType {
    CreatePost,
    AnswerPost,
    PostSolution,
    AnswerSolution,
    VoteWinner,
    WinnerAccouncement,
    Refund,
    NotYetDetermined,
    Undefined,
    EOF,
}
pub struct ProgramEvent {
    pub event_data: Vec<String>,
    pub event_type: ActionType,
    pub event_error: Option<UiTransactionError>,
}
pub async fn listen_to_program(
    program_even_chan: Sender<ProgramEvent>,
    mut cancel_chan: WatchReceiver<bool>,
) {
    let ws_url = std::env::var("WS_URL").expect("WS_URL must be set");
    let program_id = std::env::var("PROGRAM_ID").expect("PROGRAM_ID must be set");
    let client = PubsubClient::new(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut stream, unsub) = client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![program_id]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .await
        .expect("Failed to subscribe to logs");

    loop {
        tokio::select! {
            log = stream.next() => {
                if let Some(existing_log)=log{
            let content = ProgramEvent {
                event_data: existing_log.value.logs,
                event_type: ActionType::Undefined,
                event_error: existing_log.value.err,
            };
            let pos = program_even_chan.send(content).await;
            if pos.is_err() {
                break;
            }
                }
        },
         _ = cancel_chan.changed() => {
                break; //happens if is_ok() actual chancel or it does dropped, either way calcel out
                }
        }
    }

    unsub().await;
    program_even_chan
        .send(ProgramEvent {
            event_data: Vec::new(),
            event_type: ActionType::EOF,
            event_error: None,
        })
        .await;
}
