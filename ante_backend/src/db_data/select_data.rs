use sqlx::PgPool;
use tokio::sync::oneshot::Sender as oneshotSender;

use crate::db_data::postgres_runner::{
    DbCommand, PosterCreatedEventRow, SQLRequest, SQLRequestType, SQLResult, SelectDataType,
    run_request,
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
        let mut t: Vec<String> = Vec::new();
        t.push(self.id.to_string());
        t
    }
    fn get_query(&self) -> String {
        "SELECT * FROM poster_created_events (publisher, bounty_id, bounty_type, bounty_topic, bounty_minimum_gain, submission_cost, deadline, current_time, potential_answer) 
            WHERE bounty_id = $1".to_string()
    }
}

//TODO: replace the Option by proper Error
pub fn select_single_poster(
    poster_id: i32,
    info_sender: oneshotSender<Option<PosterCreatedEventRow>>,
) {
    let req = Box::new(PosterInfoGrabber {
        req_type: SQLRequestType::Select,
        id: poster_id,
    }) as Box<dyn SQLRequest>;
    let db_command: DbCommand = Box::new(move |pool: PgPool| {
        Box::pin(async move {
            let res = run_request::<PosterCreatedEventRow>(&pool, &req).await;
            if res.is_ok() {
                //TODO: more careful error handling
                let first_item = res.unwrap().items.unwrap().get(0).unwrap().clone();
                info_sender.send(Some(first_item));
            } else {
                info_sender.send(None);
            }
        })
    });
}

