use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    decryption::logs_decoder::decode_solana_logs,
    listener::{
        anchor_listener::{ActionType, ProgramEvent},
        socket_listener::WebsocketMessageCommnand,
    },
};
pub async fn read_program_logs(
    mut tx_event_receiver: Receiver<ProgramEvent>,
    websocket_chan_sender: Sender<WebsocketMessageCommnand>,
) {
    loop {
        let new_content = tx_event_receiver.recv().await;
        if let Some(response) = new_content {
            if let ActionType::EOF = response.event_type {
                break;
            }
            let mut unformatted_logs = response.event_data;
            decode_solana_logs(unformatted_logs);
            for log in response.event_data {}
        }
    }
}
