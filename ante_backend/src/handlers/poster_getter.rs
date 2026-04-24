use crate::db_data::{postgres_runner::PosterCreatedEventRow, select_data::select_single_poster};
use axum::{Json, response::IntoResponse};
use tokio::sync::oneshot::{Receiver as oneShotReceiver, Sender as oneShotSender};
#[derive(serde::Deserialize)]
pub struct GetPoster {
    id: i32,
}
pub async fn get_poster_data(Json(payload): Json<GetPoster>) -> impl IntoResponse {
    let (sender, recevier): (
        oneShotSender<Option<PosterCreatedEventRow>>,
        oneShotReceiver<Option<PosterCreatedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    select_single_poster(payload.id, sender);
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
