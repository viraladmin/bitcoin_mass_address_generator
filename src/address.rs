use crate::postgres::get_pg_client;
use tokio_postgres::Client;

pub async fn recall_address(
    seed_id: i64,
    address_index: i64,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let (client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });
    let offset = calculate_offset(address_index);
    let row = query_address(&client, seed_id, offset).await?;
    Ok(extract_address(row))
}

fn calculate_offset(address_index: i64) -> i64 {
    address_index - 1
}

fn extract_address(row: Option<tokio_postgres::Row>) -> Option<String> {
    row.map(|r| r.get(0))
}

async fn query_address(
    client: &Client,
    seed_id: i64,
    offset: i64,
) -> Result<Option<tokio_postgres::Row>, Box<dyn std::error::Error>> {
    let row = client
        .query_opt(
            "SELECT address FROM addresses WHERE seed_id = $1 ORDER BY id ASC LIMIT 1 OFFSET $2",
            &[&seed_id, &offset],
        )
        .await?;
    Ok(row)
}
