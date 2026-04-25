use sqlx::PgPool;
use tokio::sync::{mpsc::Sender, oneshot::Sender as OneShotSender};

use crate::db_data::postgres_runner::{
    AnswererDecryptedAnswerPostedEventRow, DbCommand, PosterAnsweredEventRow,
    PosterCreatedEventRow, PosterPublishAnsweredEventRow, PosterWinnerPostedEventRow,
    PublisherNotRespondedEventRow, SQLRequest, SQLRequestType, SelectDataType,
    VoteForWinnerPostedEventRow, run_request,
};

pub struct PosterInfoGrabber {
    pub req_type: SQLRequestType,
    pub id: i32,
}

impl SQLRequest for PosterInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }
    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }
    fn get_position_arg(&self) -> Vec<String> {
        vec![self.id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_created_events WHERE bounty_id = $1".to_string()
    }
}

pub struct PosterAnsweredInfoGrabber {
    pub req_type: SQLRequestType,
    pub answerer: String,
    pub poster_id: i32,
}

impl SQLRequest for PosterAnsweredInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.answerer.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_answered_events WHERE answerer = $1 AND poster_id = $2".to_string()
    }
}

pub struct AllPosterAnsweredInfoGrabber {
    pub req_type: SQLRequestType,
    pub answerer: String,
}

impl SQLRequest for AllPosterAnsweredInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.answerer.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_answered_events WHERE answerer = $1".to_string()
    }
}

pub struct PosterPublishAnsweredInfoGrabber {
    pub req_type: SQLRequestType,
    pub publisher: String,
    pub poster_id: i32,
}

impl SQLRequest for PosterPublishAnsweredInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.publisher.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_publish_answered_events WHERE publisher = $1 AND poster_id = $2"
            .to_string()
    }
}

pub struct AllPosterPublishAnsweredInfoGrabber {
    pub req_type: SQLRequestType,
    pub publisher: String,
}

impl SQLRequest for AllPosterPublishAnsweredInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.publisher.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_publish_answered_events WHERE publisher = $1".to_string()
    }
}

pub struct AnswererDecryptedAnswerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub answerer: String,
    pub poster_id: i32,
}

impl SQLRequest for AnswererDecryptedAnswerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.answerer.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM answerer_decrypted_answer_posted_events WHERE answerer = $1 AND poster_id = $2"
            .to_string()
    }
}

pub struct AllAnswererDecryptedAnswerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub answerer: String,
}

impl SQLRequest for AllAnswererDecryptedAnswerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.answerer.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM answerer_decrypted_answer_posted_events WHERE answerer = $1".to_string()
    }
}

pub struct PosterWinnerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub winner: String,
    pub poster_id: i32,
}

impl SQLRequest for PosterWinnerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.winner.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_winner_posted_events WHERE winner = $1 AND poster_id = $2"
            .to_string()
    }
}

pub struct AllPosterWinnerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub winner: String,
}

impl SQLRequest for AllPosterWinnerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.winner.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM poster_winner_posted_events WHERE winner = $1".to_string()
    }
}

pub struct PublisherNotRespondedInfoGrabber {
    pub req_type: SQLRequestType,
    pub publisher_id: String,
    pub poster_id: i32,
}

impl SQLRequest for PublisherNotRespondedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.publisher_id.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM publisher_not_responded_events WHERE publisher_id = $1 AND poster_id = $2"
            .to_string()
    }
}

pub struct AllPublisherNotRespondedInfoGrabber {
    pub req_type: SQLRequestType,
    pub publisher_id: String,
}

impl SQLRequest for AllPublisherNotRespondedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.publisher_id.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM publisher_not_responded_events WHERE publisher_id = $1".to_string()
    }
}

pub struct VoteForWinnerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub voter: String,
    pub poster_id: i32,
}

impl SQLRequest for VoteForWinnerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.voter.clone(), self.poster_id.to_string()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM vote_for_winner_posted_events WHERE voter = $1 AND poster_id = $2"
            .to_string()
    }
}

pub struct AllVoteForWinnerPostedInfoGrabber {
    pub req_type: SQLRequestType,
    pub voter: String,
}

impl SQLRequest for AllVoteForWinnerPostedInfoGrabber {
    fn get_request_type(&self) -> &SQLRequestType {
        &self.req_type
    }

    fn get_select_type(&self) -> Option<super::postgres_runner::SelectDataType> {
        Some(SelectDataType::PosterInfo)
    }

    fn get_position_arg(&self) -> Vec<String> {
        vec![self.voter.clone()]
    }

    fn get_query(&self) -> String {
        "SELECT * FROM vote_for_winner_posted_events WHERE voter = $1".to_string()
    }
}

//TODO: replace the Option by proper Error
pub fn select_single_poster(
    poster_id: i32,
    info_sender: OneShotSender<Option<PosterCreatedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(PosterInfoGrabber {
        req_type: SQLRequestType::Select,
        id: poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterCreatedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result
                        .items
                        .unwrap_or_default()
                        .into_iter()
                        .next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_poster_anwered(
    answerer: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<PosterAnsweredEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(PosterAnsweredInfoGrabber {
        req_type: SQLRequestType::Select,
        answerer,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterAnsweredEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result
                        .items
                        .unwrap_or_default()
                        .into_iter()
                        .next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_answered_post(
    answerer: String,
    info_sender: OneShotSender<Vec<PosterAnsweredEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllPosterAnsweredInfoGrabber {
        req_type: SQLRequestType::Select,
        answerer,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterAnsweredEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_poster_publish_answered(
    publisher: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<PosterPublishAnsweredEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(PosterPublishAnsweredInfoGrabber {
        req_type: SQLRequestType::Select,
        publisher,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterPublishAnsweredEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result.items.unwrap_or_default().into_iter().next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_poster_publish_answered(
    publisher: String,
    info_sender: OneShotSender<Vec<PosterPublishAnsweredEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllPosterPublishAnsweredInfoGrabber {
        req_type: SQLRequestType::Select,
        publisher,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterPublishAnsweredEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_answerer_decrypted_answer_posted(
    answerer: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<AnswererDecryptedAnswerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AnswererDecryptedAnswerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        answerer,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<AnswererDecryptedAnswerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result.items.unwrap_or_default().into_iter().next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_answerer_decrypted_answer_posted(
    answerer: String,
    info_sender: OneShotSender<Vec<AnswererDecryptedAnswerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllAnswererDecryptedAnswerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        answerer,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<AnswererDecryptedAnswerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_poster_winner_posted(
    winner: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<PosterWinnerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(PosterWinnerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        winner,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterWinnerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result.items.unwrap_or_default().into_iter().next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_poster_winner_posted(
    winner: String,
    info_sender: OneShotSender<Vec<PosterWinnerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllPosterWinnerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        winner,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterWinnerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_publisher_not_responded(
    publisher_id: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<PublisherNotRespondedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(PublisherNotRespondedInfoGrabber {
        req_type: SQLRequestType::Select,
        publisher_id,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PublisherNotRespondedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result.items.unwrap_or_default().into_iter().next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_publisher_not_responded(
    publisher_id: String,
    info_sender: OneShotSender<Vec<PublisherNotRespondedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllPublisherNotRespondedInfoGrabber {
        req_type: SQLRequestType::Select,
        publisher_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PublisherNotRespondedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_vote_for_winner_posted(
    voter: String,
    poster_id: i32,
    info_sender: OneShotSender<Option<VoteForWinnerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(VoteForWinnerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        voter,
        poster_id,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<VoteForWinnerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let first_item = result.items.unwrap_or_default().into_iter().next();
                    let _ = info_sender.send(first_item);
                }
                Err(_) => {
                    let _ = info_sender.send(None);
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}

pub fn get_all_vote_for_winner_posted(
    voter: String,
    info_sender: OneShotSender<Vec<VoteForWinnerPostedEventRow>>,
    db_request_sender: Sender<DbCommand>,
) {
    let req = Box::new(AllVoteForWinnerPostedInfoGrabber {
        req_type: SQLRequestType::Select,
        voter,
    }) as Box<dyn SQLRequest>;

    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<VoteForWinnerPostedEventRow>(&pool, &req).await;
            match res {
                Ok(result) => {
                    let rows = result.items.unwrap_or_default();
                    let _ = info_sender.send(rows);
                }
                Err(_) => {
                    let _ = info_sender.send(Vec::new());
                }
            }
        })
    });

    tokio::spawn(async move {
        let _ = db_request_sender.send(db_command).await;
    });
}
