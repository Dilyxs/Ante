use std::env;

use sqlx::{PgPool, Pool};

pub async fn create_pool(db_url: &str) -> PgPool {
    let pool = Pool::connect(db_url).await;
    //NOTE: we want to crash if we can't get it
    pool.unwrap()
}
