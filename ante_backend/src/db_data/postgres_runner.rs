use std::i32;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

use challenge_protocol::{
    AnswererDecryptedAnswerPosted, BountyTopic, BountyType, PosterAnswered, PosterCreated,
    PosterPublishAnswered, PosterWinnerPostedEvent, PublisherNotResponded, VoteForWinnerPosted,
};
use futures_util::future::BoxFuture;
use sqlx::{
    FromRow, PgPool, Pool,
    postgres::{PgQueryResult, PgRow},
};

pub async fn create_pool(db_url: &str) -> PgPool {
    let pool = Pool::connect(db_url).await;
    //NOTE: we want to crash if we can't get it
    pool.unwrap()
}
pub enum SelectDataType {
    PosterInfo,
}

#[derive(Clone, Debug)]
pub enum SQLRequestType {
    Insert,
    Update,
    Delete,
    Select,
}

pub trait SelectResult {
    fn get_content_len(&self) -> i32;
}

pub trait SQLRequest: Send + Sync {
    fn get_request_type(&self) -> &SQLRequestType;
    fn get_position_arg(&self) -> Vec<String>;
    fn get_query(&self) -> String;
    fn get_select_type(&self) -> Option<SelectDataType>;
}
pub struct RequestSQL<T> {
    chan: Sender<Result<SQLResult<T>, sqlx::Error>>,
    req: Box<dyn SQLRequest>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PosterCreatedEventRow {
    pub publisher: String,
    pub bounty_id: String,
    pub bounty_type: String,
    pub bounty_topic: String,
    pub bounty_minimum_gain: String,
    pub submission_cost: String,
    pub deadline: String,
    pub current_time: String,
    pub potential_answer: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PosterAnsweredEventRow {
    pub answerer: String,
    pub answer_time: String,
    pub poster_id: String,
    pub encrypted_answer: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PosterPublishAnsweredEventRow {
    pub publisher: String,
    pub poster_id: String,
    pub answer: String,
    pub answer_hash: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct AnswererDecryptedAnswerPostedEventRow {
    pub answerer: String,
    pub poster_id: String,
    pub answer: String,
    pub answer_hash: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PosterWinnerPostedEventRow {
    pub poster_id: String,
    pub winner: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PublisherNotRespondedEventRow {
    pub poster_id: String,
    pub publisher_id: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct VoteForWinnerPostedEventRow {
    pub poster_id: String,
    pub voter: String,
    pub winner: String,
}

fn bounty_type_to_string(bounty_type: &BountyType) -> String {
    match bounty_type {
        BountyType::OpenEnded => "open_ended".to_string(),
        BountyType::DirectAnswer => "direct_answer".to_string(),
    }
}

fn bounty_topic_to_string(bounty_topic: &BountyTopic) -> String {
    match bounty_topic {
        BountyTopic::NumberTheory => "number_theory".to_string(),
        BountyTopic::CryptoPuzzle => "crypto_puzzle".to_string(),
        BountyTopic::ReverseEng => "reverse_eng".to_string(),
        BountyTopic::NumericalTrivial => "numerical_trivial".to_string(),
        BountyTopic::PrivateKeyPuzzle => "private_key_puzzle".to_string(),
    }
}

fn option_answer_to_string(answer: &Option<[u8; 33]>) -> String {
    match answer {
        Some(value) => value
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>(),
        None => String::new(),
    }
}

impl SQLRequest for PosterCreated {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![
            self.publisher.to_string(),
            self.poster_info.bounty_id.to_string(),
            bounty_type_to_string(&self.poster_info.bounty_type),
            bounty_topic_to_string(&self.poster_info.bounty_topic),
            self.poster_info.bounty_minimum_gain.to_string(),
            self.poster_info.submission_cost.to_string(),
            self.poster_info.deadline.to_string(),
            self.poster_info.current_time.to_string(),
            option_answer_to_string(&self.poster_info.potential_answer),
        ]
    }

    fn get_query(&self) -> String {
        "INSERT INTO poster_created_events (publisher, bounty_id, bounty_type, bounty_topic, bounty_minimum_gain, submission_cost, deadline, current_time, potential_answer) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)".to_string()
    }
    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for PosterAnswered {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![
            self.answerer.to_string(),
            self.poster_response.time.to_string(),
            self.poster_response.poster_id.to_string(),
            option_answer_to_string(&self.poster_response.answer),
        ]
    }

    fn get_query(&self) -> String {
        "INSERT INTO poster_answered_events (answerer, answer_time, poster_id, encrypted_answer) VALUES ($1, $2, $3, $4)".to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for PosterPublishAnswered {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![
            self.publisher.to_string(),
            self.poster_publisher_decrypted_answer.poster_id.to_string(),
            self.poster_publisher_decrypted_answer.answer.clone(),
            self.poster_publisher_decrypted_answer.hash.clone(),
        ]
    }

    fn get_query(&self) -> String {
        "INSERT INTO poster_publish_answered_events (publisher, poster_id, answer, answer_hash) VALUES ($1, $2, $3, $4)".to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for AnswererDecryptedAnswerPosted {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![
            self.answerer.to_string(),
            self.answerer_decrypted_answer.poster_id.to_string(),
            self.answerer_decrypted_answer.answer.clone(),
            self.answerer_decrypted_answer.hash.clone(),
        ]
    }

    fn get_query(&self) -> String {
        "INSERT INTO answerer_decrypted_answer_posted_events (answerer, poster_id, answer, answer_hash) VALUES ($1, $2, $3, $4)".to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for PosterWinnerPostedEvent {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.poster_id.to_string(), self.winner.to_string()]
    }

    fn get_query(&self) -> String {
        "INSERT INTO poster_winner_posted_events (poster_id, winner) VALUES ($1, $2)".to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for PublisherNotResponded {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.poster_id.to_string(), self.published_id.to_string()]
    }

    fn get_query(&self) -> String {
        "INSERT INTO publisher_not_responded_events (poster_id, publisher_id) VALUES ($1, $2)"
            .to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

impl SQLRequest for VoteForWinnerPosted {
    fn get_request_type(&self) -> &SQLRequestType {
        &SQLRequestType::Insert
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![
            self.poster_id.to_string(),
            self.voter.to_string(),
            self.winner.to_string(),
        ]
    }

    fn get_query(&self) -> String {
        "INSERT INTO vote_for_winner_posted_events (poster_id, voter, winner) VALUES ($1, $2, $3)"
            .to_string()
    }

    fn get_select_type(&self) -> Option<SelectDataType> {
        None
    }
}

fn count_question_placeholders(query: &str) -> usize {
    query.chars().filter(|ch| *ch == '?').count()
}

fn count_postgres_placeholders(query: &str) -> usize {
    let bytes = query.as_bytes();
    let mut i = 0usize;
    let mut max_idx = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }

            if end > start
                && let Ok(idx) = query[start..end].parse::<usize>()
                && idx > max_idx
            {
                max_idx = idx;
            }

            i = end;
            continue;
        }

        i += 1;
    }

    max_idx
}

#[derive(Debug)]
pub enum SQLError {
    WrongArgCount,
}

impl std::fmt::Display for SQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SQLError::WrongArgCount => f.write_fmt(format_args!("Error WrongArgCount")),
        }
    }
}

pub struct SQLResult<T> {
    pub items: Option<Vec<T>>,
    pub pg_result: Option<PgQueryResult>,
}

impl std::error::Error for SQLError {}

pub async fn run_request<T>(
    pool: &PgPool,
    request: &Box<dyn SQLRequest>,
) -> Result<SQLResult<T>, sqlx::Error>
where
    T: std::marker::Send + Unpin + for<'a> FromRow<'a, PgRow>,
{
    let query = request.get_query();
    let args = request.get_position_arg();
    let request_type = request.get_request_type();

    let arg_count = if query.contains('?') {
        count_question_placeholders(&query)
    } else {
        count_postgres_placeholders(&query)
    };

    if args.len() != arg_count {
        return Err(sqlx::Error::Configuration(Box::new(
            SQLError::WrongArgCount,
        )));
    }
    match request_type {
        SQLRequestType::Select => {
            let mut req = sqlx::query_as::<_, T>(&query);
            for arg in args {
                req = req.bind(arg);
            }
            let rows: Vec<T> = req.fetch_all(pool).await?;
            Ok(SQLResult {
                items: Some(rows),
                pg_result: None,
            })
        }
        SQLRequestType::Insert => {
            let mut req = sqlx::query(&query);
            for arg in args {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            })
        }
        SQLRequestType::Update => {
            let mut req = sqlx::query(&query);
            for arg in args {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            })
        }
        SQLRequestType::Delete => {
            let mut req = sqlx::query(&query);
            for arg in args {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            })
        }
    }
}
pub type DbCommand = Box<dyn FnOnce(PgPool) -> BoxFuture<'static, ()> + Send>;
pub async fn run_db_requests(mut db_request_chan: Receiver<DbCommand>, pool: &PgPool) {
    while let Some(req) = db_request_chan.recv().await {
        req(pool.clone()).await;
    }
}
