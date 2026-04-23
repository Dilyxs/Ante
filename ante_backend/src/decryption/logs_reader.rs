use std::pin::Pin;

use futures_util::future::BoxFuture;
use sqlx::PgPool;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    db_data::postgres_runner::{DbCommand, SQLRequest, SQLRequestType, run_request},
    decryption::logs_decoder::decode_solana_logs,
    listener::{
        anchor_listener::{ActionType, ProgramEvent},
        socket_listener::WebsocketMessageCommnand,
    },
};
pub async fn read_program_logs(
    mut tx_event_receiver: Receiver<ProgramEvent>,
    websocket_chan_sender: Sender<WebsocketMessageCommnand>,
    sql_chan_sender: Sender<DbCommand>,
) {
    loop {
        let new_content = tx_event_receiver.recv().await;
        if let Some(response) = new_content {
            if let ActionType::EOF = response.event_type {
                break;
            }
            let unformatted_logs = response.event_data;
            for decoded_log in decode_solana_logs(unformatted_logs).into_iter() {
                if let Some(log) = decoded_log {
                    if log.websocket_command_message.is_some() {
                        let ws_chan = websocket_chan_sender.clone();
                        tokio::spawn(async move {
                            ws_chan.send(log.websocket_command_message.unwrap()).await;
                        });
                    }
                    let sql_req = log.sql_command;
                    let req: DbCommand = Box::new(move |pool: PgPool| {
                        Box::pin(async move {
                            //the request type should awalys be insert, throw other cases!
                            if let SQLRequestType::Insert = sql_req.get_request_type() {
                                run_request::<()>(&pool, &sql_req).await;
                            }
                        })
                    });
                    sql_chan_sender.send(req).await;
                };
            }
        }
    }
}
