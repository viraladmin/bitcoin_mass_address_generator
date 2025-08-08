use crate::constants::WORDLIST;
use crate::postgres::get_pg_client;
use std::fs::OpenOptions;
use std::io::Write;
use std::error::Error;

pub async fn get_write_addresses(file: &str, limit: i64) -> Result<(), Box<dyn Error>> {
    let (client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT address FROM addresses LIMIT $1", &[&limit])
        .await?;

    let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(file)?;

    for row in rows {
        let addr: &str = row.get(0);
        writeln!(f, "{}", addr)?;
    }

    Ok(())
}

pub async fn get_write_seeds(file: &str, limit: i64) -> Result<(), Box<dyn Error>> {
    let (client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT words FROM keys LIMIT $1", &[&limit])
        .await?;

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)?;

    for row in rows {
        let indices: Vec<i16> = row.get(0); // PostgreSQL array → Vec<u16>
        let phrase = indices
            .iter()
            .map(|i| {
                WORDLIST.get(*i as usize)
                    .cloned()
                    .unwrap_or_else(|| "<invalid>".to_string())
            })
            .collect::<Vec<String>>()
            .join(" ");

        writeln!(f, "{}", phrase)?;
    }

    Ok(())
}

pub async fn get_write_seeds_addresses(file: &str, limit: i64) -> Result<(), Box<dyn Error>> {
    let (client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT addresses.address, keys.words \
             FROM addresses \
             INNER JOIN keys ON addresses.seed_id = keys.id \
             LIMIT $1",
            &[&limit],
        )
        .await?;

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)?;

    for row in rows {
        let address: &str = row.get(0);
        let indices: Vec<i16> = row.get(1);

        let phrase = indices
            .iter()
            .map(|i| {
                WORDLIST.get(*i as usize)
                    .cloned()
                    .unwrap_or_else(|| "<invalid>".to_string())
            })
            .collect::<Vec<String>>()
            .join(" ");

        writeln!(f, "{} - {}", phrase, address)?;
    }

    Ok(())
}
