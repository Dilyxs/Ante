use crate::db_data::{
    postgres_runner::{
        AnswererDecryptedAnswerPostedEventRow, DbCommand, PosterAnsweredEventRow,
        PosterCreatedEventRow, PosterPublishAnsweredEventRow, PosterWinnerPostedEventRow,
        PublisherNotRespondedEventRow, VoteForWinnerPostedEventRow,
    },
    select_data::{
        get_all_answered_post, get_all_answerer_decrypted_answer_posted,
        get_all_poster_publish_answered, get_all_poster_winner_posted,
        get_all_publisher_not_responded, get_all_vote_for_winner_posted,
        get_answerer_decrypted_answer_posted, get_poster_anwered, get_poster_publish_answered,
        get_poster_winner_posted, get_publisher_not_responded, get_vote_for_winner_posted,
        select_single_poster,
    },
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

#[derive(serde::Deserialize)]
pub struct GetPosterPublishAnswered {
    publisher: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllPosterPublishAnswered {
    publisher: String,
}

#[derive(serde::Deserialize)]
pub struct GetAnswererDecryptedAnswerPosted {
    answerer: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllAnswererDecryptedAnswerPosted {
    answerer: String,
}

#[derive(serde::Deserialize)]
pub struct GetPosterWinnerPosted {
    winner: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllPosterWinnerPosted {
    winner: String,
}

#[derive(serde::Deserialize)]
pub struct GetPublisherNotResponded {
    publisher_id: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllPublisherNotResponded {
    publisher_id: String,
}

#[derive(serde::Deserialize)]
pub struct GetVoteForWinnerPosted {
    voter: String,
    poster_id: i32,
}

#[derive(serde::Deserialize)]
pub struct GetAllVoteForWinnerPosted {
    voter: String,
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

pub async fn get_poster_publish_answered_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetPosterPublishAnswered>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<PosterPublishAnsweredEventRow>>,
        oneShotReceiver<Option<PosterPublishAnsweredEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_poster_publish_answered(
        payload.publisher,
        payload.poster_id,
        sender,
        db_request_sender,
    );

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving poster publish answered from db thread: {:?}", err);
            Json(None)
        }
    }
}

pub async fn get_all_poster_publish_answered_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllPosterPublishAnswered>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<PosterPublishAnsweredEventRow>>,
        oneShotReceiver<Vec<PosterPublishAnsweredEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_poster_publish_answered(payload.publisher, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving all poster publish answered from db thread: {:?}",
                err
            );
            Json(Vec::<PosterPublishAnsweredEventRow>::new())
        }
    }
}

pub async fn get_answerer_decrypted_answer_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAnswererDecryptedAnswerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<AnswererDecryptedAnswerPostedEventRow>>,
        oneShotReceiver<Option<AnswererDecryptedAnswerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_answerer_decrypted_answer_posted(
        payload.answerer,
        payload.poster_id,
        sender,
        db_request_sender,
    );

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving answerer decrypted answer posted from db thread: {:?}",
                err
            );
            Json(None)
        }
    }
}

pub async fn get_all_answerer_decrypted_answer_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllAnswererDecryptedAnswerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<AnswererDecryptedAnswerPostedEventRow>>,
        oneShotReceiver<Vec<AnswererDecryptedAnswerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_answerer_decrypted_answer_posted(payload.answerer, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving all answerer decrypted answer posted from db thread: {:?}",
                err
            );
            Json(Vec::<AnswererDecryptedAnswerPostedEventRow>::new())
        }
    }
}

pub async fn get_poster_winner_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetPosterWinnerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<PosterWinnerPostedEventRow>>,
        oneShotReceiver<Option<PosterWinnerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_poster_winner_posted(payload.winner, payload.poster_id, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving poster winner posted from db thread: {:?}", err);
            Json(None)
        }
    }
}

pub async fn get_all_poster_winner_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllPosterWinnerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<PosterWinnerPostedEventRow>>,
        oneShotReceiver<Vec<PosterWinnerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_poster_winner_posted(payload.winner, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving all poster winner posted from db thread: {:?}",
                err
            );
            Json(Vec::<PosterWinnerPostedEventRow>::new())
        }
    }
}

pub async fn get_publisher_not_responded_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetPublisherNotResponded>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<PublisherNotRespondedEventRow>>,
        oneShotReceiver<Option<PublisherNotRespondedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_publisher_not_responded(
        payload.publisher_id,
        payload.poster_id,
        sender,
        db_request_sender,
    );

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving publisher not responded from db thread: {:?}", err);
            Json(None)
        }
    }
}

pub async fn get_all_publisher_not_responded_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllPublisherNotResponded>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<PublisherNotRespondedEventRow>>,
        oneShotReceiver<Vec<PublisherNotRespondedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_publisher_not_responded(payload.publisher_id, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving all publisher not responded from db thread: {:?}",
                err
            );
            Json(Vec::<PublisherNotRespondedEventRow>::new())
        }
    }
}

pub async fn get_vote_for_winner_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetVoteForWinnerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Option<VoteForWinnerPostedEventRow>>,
        oneShotReceiver<Option<VoteForWinnerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_vote_for_winner_posted(payload.voter, payload.poster_id, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!("Error receiving vote for winner posted from db thread: {:?}", err);
            Json(None)
        }
    }
}

pub async fn get_all_vote_for_winner_posted_data(
    State(db_request_sender): State<Sender<DbCommand>>,
    Json(payload): Json<GetAllVoteForWinnerPosted>,
) -> impl IntoResponse {
    let (sender, receiver): (
        oneShotSender<Vec<VoteForWinnerPostedEventRow>>,
        oneShotReceiver<Vec<VoteForWinnerPostedEventRow>>,
    ) = tokio::sync::oneshot::channel();
    get_all_vote_for_winner_posted(payload.voter, sender, db_request_sender);

    match receiver.await {
        Ok(content) => Json(content),
        Err(err) => {
            println!(
                "Error receiving all vote for winner posted from db thread: {:?}",
                err
            );
            Json(Vec::<VoteForWinnerPostedEventRow>::new())
        }
    }
}
