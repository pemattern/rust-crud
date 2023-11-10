use sqlx::Connection;
use sqlx::Row;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://postgres:blackdahlia1993@localhost:5432/rustcrud";
    let mut conn = sqlx::postgres::PgConnection::connect(url).await?;

    let res = sqlx::query(
        "SELECT id, email FROM users WHERE password = crypt('blackdahlia1993', password)",
    )
    .fetch_one(&mut conn)
    .await?;
    let id: i64 = res.get("id");
    let email: &str = res.get("email");

    println!("{}, {}", id, email);
    Ok(())
}
