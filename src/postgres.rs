use bytes::Bytes;
use futures::{pin_mut, SinkExt};
use std::error::Error;
use tokio_postgres::{Client, Connection, NoTls, Socket};
use tokio_postgres::tls::NoTlsStream;

pub async fn get_pg_client() -> Result<(Client, Connection<Socket, NoTlsStream>), Box<dyn Error>> {
    let db_url = dotenvy::var("DATABASE_URL")?; // ✅ this uses the crate, no warning

    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;
    Ok((client, connection))
}


pub async fn write_to_db<'a>(
    client: &'a mut Client,
    all_keys: &'a [(i64, Vec<i16>)],
    all_addresses: &'a [(i64, i64, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let txn = client.transaction().await?;

    {
        let sink = txn.copy_in("COPY keys (id, words) FROM STDIN").await?;
        pin_mut!(sink);

        for (id, words) in all_keys {
            let line = format!("{}\t{{{}}}\n", id, words.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(","));
            sink.send(Bytes::from(line)).await?;
        }

        sink.close().await?;
    }

    {
        let sink = txn.copy_in("COPY addresses (id, seed_id, address) FROM STDIN").await?;
        pin_mut!(sink);

        for (address_id, seed_id, address) in all_addresses {
            let line = format!("{}\t{}\t{}\n", address_id, seed_id, address);
            sink.send(Bytes::from(line)).await?;
        }

        sink.close().await?;
    }
    txn.commit().await?;
    Ok(())
}
