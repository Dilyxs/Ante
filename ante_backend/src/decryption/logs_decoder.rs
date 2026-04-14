use crate::listener::socket_listener::WebsocketMessageCommnand;
use crate::{BlockchainEvent, listener::socket_listener::ResponseToWebSocket};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, Discriminator};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use challenge_protocol::{
    AnswererDecryptedAnswerPosted, PosterAnswered, PosterCreated, PosterPublishAnswered,
    PosterWinnerPostedEvent, PublisherNotResponded, VoteForWinnerPosted,
};
use std::convert::TryInto;

pub fn decode_solana_logs(input: Vec<String>) -> Option<WebsocketMessageCommnand> {
    // Collect decoded program-data logs (base64 -> bytes)
    let emitted_logs: Vec<Result<Vec<u8>, base64::DecodeError>> = input
        .iter()
        .filter(|x| x.contains("Program data:"))
        .map(|x| STANDARD.decode(x))
        .collect();

    // Iterate through decoded logs and try to match known discriminators.
    for log in emitted_logs {
        if let Ok(bytes) = log {
            if bytes.len() < 8 {
                continue;
            }

            let (disc_slice, payload) = bytes.split_at(8);
            let disc: [u8; 8] = match disc_slice.try_into() {
                Ok(d) => d,
                Err(_) => continue,
            };

            let mut payload_slice = &payload[..];

            if disc == PosterCreated::DISCRIMINATOR {
                if let Ok(content) = PosterCreated::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewPost),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == PosterAnswered::DISCRIMINATOR {
                if let Ok(content) = PosterAnswered::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewAnswer),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == PosterPublishAnswered::DISCRIMINATOR {
                if let Ok(content) = PosterPublishAnswered::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewAnswer),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == AnswererDecryptedAnswerPosted::DISCRIMINATOR {
                if let Ok(content) = AnswererDecryptedAnswerPosted::deserialize(&mut payload_slice)
                {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewAnswer),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == PosterWinnerPostedEvent::DISCRIMINATOR {
                if let Ok(content) = PosterWinnerPostedEvent::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewWinner),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == PublisherNotResponded::DISCRIMINATOR {
                if let Ok(content) = PublisherNotResponded::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewPost),
                        log_info: Some(Box::new(content)),
                    });
                }
            } else if disc == VoteForWinnerPosted::DISCRIMINATOR {
                if let Ok(content) = VoteForWinnerPosted::deserialize(&mut payload_slice) {
                    return Some(WebsocketMessageCommnand {
                        message_type: None,
                        user_channel: None,
                        user_id: None,
                        block_chain_event: Some(BlockchainEvent::NewVote),
                        log_info: Some(Box::new(content)),
                    });
                }
            }
        }
    }

    None
}

use crate::listener::socket_listener::EmitLog;

impl EmitLog for PosterAnswered {}
impl EmitLog for PosterPublishAnswered {}
impl EmitLog for AnswererDecryptedAnswerPosted {}
impl EmitLog for PosterWinnerPostedEvent {}
impl EmitLog for PublisherNotResponded {}
