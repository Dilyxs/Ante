use sqlx::PgPool;
use tokio::sync::{mpsc::Sender, oneshot::Sender as OneShotSender};

use crate::db_data::postgres_runner::{
    DbCommand, PosterAnsweredEventRow, PosterCreatedEventRow, SQLRequest, SQLRequestType,
    SelectDataType, run_request,
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
