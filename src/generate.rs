use bip39::{Language, Mnemonic};
use bitcoin::{ Address, Network, XOnlyPublicKey };
use bitcoin::bip32::{ ChildNumber, DerivationPath, Xpriv, Xpub };
use bitcoin::key::Secp256k1;
use crate::constants::WORDLIST;
use crate::postgres::{get_pg_client, write_to_db};
use rand::seq::SliceRandom;
use std::str::FromStr;


fn generate_mnemonic() -> Mnemonic {
    loop {
        let candidate_words: Vec<String> = WORDLIST
            .choose_multiple(&mut rand::thread_rng(), 12)
            .map(|w| w.to_string())
            .collect();

        if let Ok(m) = Mnemonic::parse_in(Language::English, &candidate_words.join(" ")) {
            return m;
        }
    }
}

fn get_word_indexes(seed_phrase: &str) -> Vec<i16> {
    seed_phrase
        .split_whitespace()
        .map(|w| WORDLIST.iter().position(|x| *x == w).unwrap() as i16)
        .collect()
}

fn derive_xpubs(master_xprv: &Xpriv, secp: &Secp256k1<bitcoin::secp256k1::All>) -> Result<Vec<Xpub>, Box<dyn std::error::Error>> {
    let paths = ["m/44'/0'/0'/0", "m/49'/0'/0'/0", "m/84'/0'/0'/0", "m/86'/0'/0'/0"];
    let mut xpubs = Vec::new();
    for path_str in paths {
        let path = DerivationPath::from_str(path_str)?;
        let xprv = master_xprv.derive_priv(secp, &path)?;
        xpubs.push(Xpub::from_priv(secp, &xprv));
    }
    Ok(xpubs)
}

fn get_enabled_types_from_env() -> Result<Vec<&'static str>, Box<dyn std::error::Error>> {
    let raw = dotenvy::var("TYPES").unwrap_or_default();

    let mapping = [
        ("legacy", "Legacy P2PKH (BIP44)"),
        ("segwit", "P2SH-P2WPKH (BIP49)"),
        ("segwit_native", "Native SegWit P2WPKH (BIP84)"),
        ("taproot", "Taproot P2TR (BIP86)"),
    ];

    let selected = raw
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter_map(|key| {
            mapping.iter().find(|(k, _)| *k == key).map(|(_, label)| *label)
        })
        .collect();

    Ok(selected)
}


fn generate_addresses(
    seed_index: i64,
    base_xpubs: &[Xpub],
    secp: &Secp256k1<bitcoin::secp256k1::All>,
) -> Result<Vec<(i64, i64, String)>, Box<dyn std::error::Error>> {

    let types = get_enabled_types_from_env()?;
    let capacity: usize = dotenvy::var("ADDRESSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let mut addresses = Vec::with_capacity(capacity);
    let loop_amount: usize = capacity / types.len();

    for (idx, label) in types.iter().enumerate() {
        for index in 0..loop_amount {
            let address_index = ((index + 1) + (idx * loop_amount)) as i64;
            let child_number = ChildNumber::Normal { index: index as u32 };
            let child_xpub = base_xpubs[idx].derive_pub(secp, &[child_number])?;
            let btc_pubkey = bitcoin::PublicKey {
                inner: child_xpub.public_key,
                compressed: true,
            };

            let address = match *label {
                "Legacy P2PKH (BIP44)" => Address::p2pkh(&btc_pubkey, Network::Bitcoin),
                "P2SH-P2WPKH (BIP49)" => Address::p2shwpkh(&btc_pubkey, Network::Bitcoin)?,
                "Native SegWit P2WPKH (BIP84)" => Address::p2wpkh(&btc_pubkey, Network::Bitcoin)?,
                "Taproot P2TR (BIP86)" => {
                    let xonly = XOnlyPublicKey::from(child_xpub.public_key);
                    Address::p2tr(secp, xonly, None, Network::Bitcoin)
                }
                _ => unreachable!(),
            };
            addresses.push((address_index, seed_index, address.to_string()));
        }
    }

    Ok(addresses)
}

pub async fn generate(
    range_start: usize,
    range_end: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, connection) = get_pg_client().await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let writes: usize = dotenvy::var("WRITES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let capacity: usize = dotenvy::var("ADDRESSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let true_copacity = writes * capacity;

    let secp = Secp256k1::new();
    let mut counter = 0;
    let mut all_addresses = Vec::with_capacity(true_copacity);
    let mut all_keys = Vec::with_capacity(writes);

    for i in range_start..range_end {
        let seed_index = (i + 1) as i64;
        if client.query_opt("SELECT 1 FROM keys WHERE id = $1", &[&seed_index]).await?.is_some() {
            println!("skipping index: {}", &seed_index);
            continue;
        }

        let mnemonic = generate_mnemonic();
        let seed_words = mnemonic.to_string();
        let word_indexes = get_word_indexes(&seed_words);
        all_keys.push((seed_index, word_indexes));

        let seed_bytes = mnemonic.to_seed("");
        let master_xprv = Xpriv::new_master(Network::Bitcoin, &seed_bytes)?;
        let base_xpubs = derive_xpubs(&master_xprv, &secp)?;
        let addresses = generate_addresses(seed_index, &base_xpubs, &secp)?;
        all_addresses.extend(addresses);

        counter += 1;
        if counter == writes {
            println!("writing");
            write_to_db(&mut client, &all_keys, &all_addresses).await?;
            all_keys.clear();
            all_addresses.clear();
            counter = 0;
        }
    }

    Ok(())
}
