use dotenvy::dotenv;

mod address;
use address::recall_address;

mod constants;

mod generate;
use generate::generate;

mod postgres;

mod seeds;
use seeds::recall_seeds;
use seeds::generate_keys;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  generate");
        eprintln!("  recall <seed_index> <address_index>");
        return Ok(());
    }

    match args[1].as_str() {
        "generate" => {
            drop(args);
            dotenv().ok();
            let num_tasks: i64 = dotenvy::var("THREADS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let total_seeds: i64 = dotenvy::var("SEEDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let chunk_size = (total_seeds + num_tasks - 1) / num_tasks;

            let mut handles = Vec::new();
            for task_id in 0..num_tasks {
                let start = task_id * chunk_size;
                let end = if task_id == num_tasks - 1 {
                    total_seeds
                } else {
                    start + chunk_size
                };
                let handle = tokio::spawn(async move {
                    println!("start: {}, end: {}", &start, &end);
                    if let Err(e) = generate(start as usize, end as usize).await {
                        eprintln!("Task {} error: {}", task_id, e);
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.await.unwrap();
            }
        }
        "recall" => {
            if args.len() != 4 {
                eprintln!("Usage: recall <seed_index> <address_index>");
                return Ok(());
            }
            let seed_index: u64 = args[2].parse()?;
            let address_index: u64 = args[3].parse()?;
            let mut seed_words = String::new();
            let mut wallet_address = String::new();
            match recall_seeds(seed_index as i64).await {
                Ok(Some(seed_phrase)) => {
                    seed_words = seed_phrase.clone();
                    println!("Seed phrase: {}", seed_phrase);
                }
                Ok(None) => {
                    println!("Seed phrase not found for index {}", seed_index);
                }
                Err(e) => {
                    eprintln!("Error recalling seed: {}", e);
                }
            }

            match recall_address(seed_index as i64, address_index as i64).await {
                Ok(Some(address)) => {
                    wallet_address = address.clone();
                    println!("Address: {}", address);
                }
                Ok(None) => {
                    println!("Address not found for seed {} address {}", seed_index, address_index);
                }
                Err(e) => {
                    eprintln!("Error recalling address: {}", e);
                }
            }

            if let Some((wif_c, wif_u, mini, raw)) = generate_keys(
                &seed_words,
                address_index,
                &wallet_address,
            ) {
                println!("✅ WIF (compressed):     {}", wif_c);
                println!("✅ WIF (uncompressed):   {}", wif_u);
                println!("✅ Mini private key:     {}", mini);
                println!("✅ Raw hex private key:  {}", raw);
            } else {
                println!("❌ Could not generate keys.");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Commands:");
            eprintln!("  generate");
            eprintln!("  recall <seed_index> <address_index>");
        }
    }
    Ok(())
}
