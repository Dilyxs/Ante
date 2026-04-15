use sqlx::{
    FromRow, PgPool, Pool,
    postgres::{PgQueryResult, PgRow},
};

pub async fn create_pool(db_url: &str) -> PgPool {
    let pool = Pool::connect(db_url).await;
    //NOTE: we want to crash if we can't get it
    pool.unwrap()
}
pub enum SQLRequestType {
    Insert,
    Update,
    Delete,
    Select,
}
pub trait SQLRequest<T> {
    fn get_request_type(&self) -> SQLRequestType;
    fn get_position_arg(&self) -> Vec<String>;
    fn get_query(&self) -> String;
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
    request: &dyn SQLRequest<T>,
) -> Result<SQLResult<T>, sqlx::Error>
where
    T: std::marker::Send + Unpin + for<'a> FromRow<'a, PgRow>,
{
    let query = request.get_query();
    let args = request.get_position_arg();
    let request_type = request.get_request_type();
    let arg_count = query
        .chars()
        .fold(0, |acc, x| if x == '?' { acc + 1 } else { acc });
    if args.len() != arg_count {
        return Err(sqlx::Error::Configuration(Box::new(
            SQLError::WrongArgCount,
        )));
    }
    match request_type {
        SQLRequestType::Select => {
            let mut req = sqlx::query_as::<_, T>(&query);
            for arg in args.into_iter() {
                req = req.bind(arg);
            }
            let r: Vec<T> = req.fetch_all(pool).await?;
            return Ok(SQLResult {
                items: Some(r),
                pg_result: None,
            });
        }
        SQLRequestType::Insert => {
            let mut req = sqlx::query(&query);
            for arg in args.into_iter() {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            return Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            });
        }
        SQLRequestType::Update => {
            let mut req = sqlx::query(&query);
            for arg in args.into_iter() {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            return Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            });
        }
        SQLRequestType::Delete => {
            let mut req = sqlx::query(&query);
            for arg in args.into_iter() {
                req = req.bind(arg);
            }
            let res = req.execute(pool).await?;
            return Ok(SQLResult {
                items: None,
                pg_result: Some(res),
            });
        }
    }
}
