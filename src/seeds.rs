use bitcoin::{
    bip32::{DerivationPath, Xpriv},
    Network, PrivateKey,
    secp256k1::Secp256k1,
};
use bip39::{Mnemonic, Language};
use crate::constants::WORDLIST;
use crate::postgres::get_pg_client;
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};
use std::error::Error;
use std::str::FromStr;
use tokio_postgres::Client;


async fn query_seed_indices(client: &Client, seed_id: i64) -> Result<Option<Vec<i16>>, Box<dyn Error>> {
    let row = client
        .query_opt("SELECT words FROM keys WHERE id = $1", &[&seed_id])
        .await?;

    Ok(row.map(|r| r.get(0)))
}

fn indices_to_phrase(wordlist: &[String], indices: &[i16]) -> String {
    indices
        .iter()
        .map(|&i| wordlist.get(i as usize).map(|s| s.as_str()).unwrap_or("<unknown>"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn recall_seeds(seed_id: i64) -> Result<Option<String>, Box<dyn Error>> {
    let (client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let indices = query_seed_indices(&client, seed_id).await?;

    if let Some(indices) = indices {
        let phrase = indices_to_phrase(&WORDLIST, &indices);
        Ok(Some(phrase))
    } else {
        Ok(None)
    }
}

pub fn generate_keys(
    seed_phrase: &str,
    index: u64,
    target_address: &str,
) -> Option<(String, String, String, String)> {
    let mnemonic = Mnemonic::parse_in(Language::English, seed_phrase).ok()?;
    let seed = mnemonic.to_seed_normalized("");
    let network = Network::Bitcoin;

    let secp = Secp256k1::new();
    let xprv = Xpriv::new_master(network, &seed).ok()?;

    let path_prefix = if target_address.starts_with('1') {
        "m/44'/0'/0'/0"
    } else if target_address.starts_with('3') {
        "m/49'/0'/0'/0"
    } else if target_address.starts_with("bc1q") {
        "m/84'/0'/0'/0"
    } else if target_address.starts_with("bc1p") {
        "m/86'/0'/0'/0"
    } else {
        return None;
    };

    let full_path = format!("{}/{}", path_prefix, index);
    let derivation_path = DerivationPath::from_str(&full_path).ok()?;
    let child_xprv = xprv.derive_priv(&secp, &derivation_path).ok()?;

    let secret_key = child_xprv.private_key;

    // Compressed WIF
    let priv_compressed = PrivateKey {
        network,
        compressed: true,
        inner: secret_key,
    };
    let wif_compressed = priv_compressed.to_wif();

    // Uncompressed WIF
    let priv_uncompressed = PrivateKey {
        network,
        compressed: false,
        inner: secret_key,
    };
    let wif_uncompressed = priv_uncompressed.to_wif();

    // Raw hex
    let raw_hex = hex::encode(secret_key.secret_bytes());

    // Mini key (try until SHA256(mini)[0] == 0x00)
    let mut rng = thread_rng();
    for _ in 0..10_000 {
        let mini_candidate: String = {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789";
            let mut candidate = String::from("S");
            for _ in 0..21 {
                candidate.push(chars[rng.gen_range(0..chars.len())] as char);
            }
            candidate
        };

        let mut hasher = Sha256::new();
        hasher.update(mini_candidate.as_bytes());
        if hasher.finalize()[0] == 0x00 {
            return Some((
                wif_compressed,
                wif_uncompressed,
                mini_candidate,
                raw_hex,
            ));
        }
    }

    Some((
        wif_compressed,
        wif_uncompressed,
        "[failed to generate mini key]".to_string(),
        raw_hex,
    ))
}
