use crate::postgres::{ get_pg_client, get_pg_client2, };
use std::error::Error;


pub async fn examine_addresses(seed: i64) -> Result<Vec<String>, Box<dyn Error>> {
    // Connect to DB1 (addresses)
    let (client1, connection1) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection1.await {
            eprintln!("DB1 Postgres connection error: {}", e);
        }
    });

    // Connect to DB2 (wallet_balances)
    let (client2, connection2) = get_pg_client2().await?;
    tokio::spawn(async move {
        if let Err(e) = connection2.await {
            eprintln!("DB2 Postgres connection error: {}", e);
        }
    });

    // Query addresses from DB1 for this seed
    let rows = client1
        .query("SELECT address FROM addresses WHERE seed_id = $1", &[&seed])
        .await?;

    let mut matches = Vec::new();

    for row in rows {
        let address: String = row.get("address");

        // Check if the address exists in DB2
        let exists = client2
            .query_opt(
                "SELECT 1 FROM wallet_balances WHERE wallet_address = $1",
                &[&address],
            )
            .await?;

        if exists.is_some() {
            matches.push(address);
        }
    }

    Ok(matches)
}
