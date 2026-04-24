use crate::db_data::{
    postgres_runner::{DbCommand, PosterAnsweredEventRow, PosterCreatedEventRow},
    select_data::{get_all_answered_post, get_poster_anwered, select_single_poster},
};
use axum::{Json, extract::State, response::IntoResponse};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{Receiver as oneShotReceiver, Sender as oneShotSender};

#[derive(serde::Deserialize)]
pub struct GetPoster {
    id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetPosterAnswered {
    answerer: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllAnsweredPost {
    answerer: String,
}

pub async fn get_poster_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetPoster>,
) -> impl IntoResponse {
    let (sender, recevier): (
        oneShotSender<Option<PosterCreatedEventRow>>,
        oneShotReceiver<Option<PosterCreatedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    select_single_poster(payload.id, sender, db_request_sender);
    //now we wait for the data to be sent back from the db thread
    let res = recevier.await;
    match res {
        Ok(no_err) => {
            if no_err.is_some() {
                return Json(no_err);
            } else {
                println!("Error: No data found for poster with id {}", payload.id);
                return Json(None);
            }
        }
        Err(err) => {
            println!("Error receiving data from db thread: {:?}", err);
            return Json(None);
        }
    }
}

pub async fn get_poster_answered_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetPosterAnswered>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<PosterAnsweredEventRow>>,
        oneShotReceiver<Option<PosterAnsweredEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_poster_anwered(payload.answerer, payload.poster_id, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving answered post from db thread: {:?}", err);
            Json(None)
        }
    }
}

pub async fn get_all_answered_post_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllAnsweredPost>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<PosterAnsweredEventRow>>,
        oneShotReceiver<Vec<PosterAnsweredEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_answered_post(payload.answerer, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving answered posts from db thread: {:?}", err);
            Json(Vec::<PosterAnsweredEventRow>::new())
        }
    }
}
