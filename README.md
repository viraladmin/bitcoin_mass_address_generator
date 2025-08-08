# bitcoin_mass_address_generator

A high-performance, multithreaded Bitcoin HD wallet address generator and seed manager written in Rust. Supports BIP44, BIP49, BIP84, and BIP86 derivation paths. Designed for research, auditing, and bulk key generation at massive scale — billions of addresses per hour on modern hardware.

---

## Show Support

I make nothing creating and sharing the tools I develop. I do it for my love of the space and the people in it.

Help a fellow dev out, I ain't vibe coding here. What's a sat or two between friends? :)

Bitcoin: bc1ql9xt4l62ly6pp7s9358rkdefrwc0mm5yne78xl

---

## Features

- Generates valid BIP39 seed phrases (12-word)
- Derives multiple BIP32 paths:
  - Legacy P2PKH (BIP44)
  - SegWit nested P2SH-P2WPKH (BIP49)
  - Native SegWit P2WPKH (BIP84)
  - Taproot P2TR (BIP86)
- Uses PostgreSQL for fast, concurrent bulk insert of:
  - Seed → wordlist index mapping
  - Wallet addresses
- Recalls addresses, seedwords and private keys from seed index and address index
- Writes addresses and seeds to file
- Examines addresses for balance (requires [Bitcoin Balance ETL](https://github.com/viraladmin/bitcoin_balance_etl))

---

## Requirements

- Rust 1.76+
- PostgreSQL
- Linux (for best performance; WSL has known memory limits)

---

## Installation

```bash
cargo install bitcoin_mass_address_generator
```

---

## Configuration

Create a .env file in your root directory:

```env
# PostgreSQL connection string
DATABASE_URL="postgres://user:pass@localhost:5432/wallet_addresses"

# PostgreSQL connection string for [Bitcoin Balance ETL](https://github.com/viraladmin/bitcoin_balance_etl)
# Leave empty if not installed.
# DATABASE_URL2="postgres://user:pass@localhost:5432/btc_wallets"
DATABASE_URL2=""

# Address types to generate (comma-separated)
# Valid: legacy, segwit, segwit_native, taproot
TYPES="legacy, segwit, segwit_native, taproot"

# Total number of threads to run (parallel tasks)
THREADS=20

# Number of seeds to generate
SEEDS=1000000

# Number of addresses per seed
ADDRESSES=4000000

# How often to flush to DB (e.g. 2 = write every 2 seeds)
WRITES=2
```

---

## PostgreSQL Schema

```sql
CREATE TABLE keys (
    id BIGINT,
    words SMALLINT[] NOT NULL
);

CREATE TABLE addresses (
    id BIGINT,
    seed_id BIGINT NOT NULL,
    address TEXT NOT NULL
);
```

---

## After Address Creation

Add indexes to the database tables after creation.

```sql
CREATE INDEX ON addresses(seed_id, id);
CREATE INDEX ON keys(id);
```

---

## Usage

### Generate Addresses

```bash
./bitcoin_mass_address_generator generate
```

This will spawn N threads (defined in .env) to generate seed phrases and addresses, writing in bulk to PostgreSQL.

---

### Recall Wallet Data

```bash
./bitcoin_mass_address_generator recall <seed_index> <address_index>
```

Returns:

- Seed Phrase
- Address
- Private Key
  - WIF (compressed)
  - WIF (uncompressed)
  - Raw hex
  - Mini key (if derivable)

---

## Write Data to File

```bash
./bitcoin_mass_address_generator write_file <option> <file> <limit>
```

Exports data from the database to a file. Each option controls what is written.

#### Options:

- `addresses`  
  Writes a list of wallet addresses (1 per line).  
  
  Example output:
    bc1qxyz...
    bc1qabc...

- `seeds`  
  Writes seed phrases (1 per line). These are human-readable phrases derived from stored index arrays.  
  
  Example output:
    gravity spy pencil vault signal elite brief avoid coach style flush attack  

- `seeds_addresses`

  Writes seed phrase and corresponding address on each line.
  
  Example output:
    gravity spy pencil vault signal elite brief avoid coach style flush attack - bc1qxyz...

#### Parameters:

- `<file>` — Output filename  
- `<limit>` — Max number of rows to export

#### Example:

```bash
./bitcoin_mass_address_generator write_file seeds_addresses exported.txt 100
```

This will write the first 100 seed-address pairs to exported.txt.

---

## Examine Addresses

```bash
./bitcoin_mass_address_generator examine <seed_id>
```

Checks each address under given seed id and returns address if balance is greater than 0.

#### Example:

```bash
./bitcoin_mass_address_generator examine 1
```

Returns all addresses for the first seed with a balance greater than 0.

---

## Performance Notes
- Designed for high-speed environments
- Works best on real Linux (not WSL)
- PostgreSQL COPY is used for fast bulk inserts
- Memory tuning may be needed for large volumes

---

## License

MIT

---

## Disclaimer

This tool is provided as is for educational and research purposes only. Do not use it to generate production wallets without extreme care. No warranty is provided for any loss of funds.
