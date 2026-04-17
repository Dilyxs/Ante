use crate::BlockchainEvent;
use crate::db_data::postgres_runner::SQLRequest;
use crate::listener::socket_listener::{
    NewPost, NewVote, NewWinner, RecentAnswer, WebsocketMessageCommnand,
};
use anchor_lang::{AnchorDeserialize, Discriminator};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use challenge_protocol::{
    AnswererDecryptedAnswerPosted, PosterAnswered, PosterCreated, PosterPublishAnswered,
    PosterWinnerPostedEvent, PublisherNotResponded, VoteForWinnerPosted,
};
use std::convert::TryInto;

pub struct LogInfo {
    pub websocket_command_message: Option<WebsocketMessageCommnand>,
    pub sql_command: Box<dyn SQLRequest>,
}

fn to_program_data_bytes(line: &str) -> Option<Vec<u8>> {
    let encoded_payload = line.split("Program data:").nth(1)?.trim();
    STANDARD.decode(encoded_payload).ok()
}

pub fn decode_solana_logs(input: Vec<String>) -> Vec<Option<LogInfo>> {
    let mut decoded_logs: Vec<Option<LogInfo>> = Vec::new();
    for line in input {
        if !line.contains("Program data:") {
            continue;
        }

        let bytes = match to_program_data_bytes(&line) {
            Some(value) => value,
            None => continue,
        };

        if bytes.len() < 8 {
            continue;
        }

        let (disc_slice, payload) = bytes.split_at(8);
        let disc: [u8; 8] = match disc_slice.try_into() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let mut payload_slice = &payload[..];

        if disc == PosterCreated::DISCRIMINATOR {
            if let Ok(content) = PosterCreated::deserialize(&mut payload_slice) {
                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewPost),
                        log_info: Some(Box::new(NewPost::new(content.clone()))),
                    }),
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == PosterAnswered::DISCRIMINATOR {
            if let Ok(content) = PosterAnswered::deserialize(&mut payload_slice) {
                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: None,
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == PosterPublishAnswered::DISCRIMINATOR {
            if let Ok(content) = PosterPublishAnswered::deserialize(&mut payload_slice) {
                let response = RecentAnswer::new(
                    true,
                    true,
                    content.poster_publisher_decrypted_answer.answer.clone(),
                    content.poster_publisher_decrypted_answer.hash.clone(),
                    content.poster_publisher_decrypted_answer.poster_id as i32,
                );

                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewAnswer),
                        log_info: Some(Box::new(response)),
                    }),
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == AnswererDecryptedAnswerPosted::DISCRIMINATOR {
            if let Ok(content) = AnswererDecryptedAnswerPosted::deserialize(&mut payload_slice) {
                let response = RecentAnswer::new(
                    true,
                    false,
                    content.answerer_decrypted_answer.answer.clone(),
                    content.answerer_decrypted_answer.hash.clone(),
                    content.answerer_decrypted_answer.poster_id as i32,
                );

                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewAnswer),
                        log_info: Some(Box::new(response)),
                    }),
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == PosterWinnerPostedEvent::DISCRIMINATOR {
            if let Ok(content) = PosterWinnerPostedEvent::deserialize(&mut payload_slice) {
                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewWinner),
                        log_info: Some(Box::new(NewWinner::new(content.clone()))),
                    }),
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == PublisherNotResponded::DISCRIMINATOR {
            if let Ok(content) = PublisherNotResponded::deserialize(&mut payload_slice) {
                decoded_logs.push(Some(LogInfo {
                    websocket_command_message: None,
                    sql_command: Box::new(content),
                }));
            }
        } else if disc == VoteForWinnerPosted::DISCRIMINATOR
            && let Ok(content) = VoteForWinnerPosted::deserialize(&mut payload_slice)
        {
            decoded_logs.push(Some(LogInfo {
                websocket_command_message: Some(WebsocketMessageCommnand {
                    message_type: None,
                    user_channel: None,
                    user_id: None,
                    block_chain_event: Some(BlockchainEvent::NewVote),
                    log_info: Some(Box::new(NewVote::new(content.clone()))),
                }),
                sql_command: Box::new(content),
            }));
        }
    }

    decoded_logs
}
