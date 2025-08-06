# bitcoin_mass_address_generator

A high-performance, multithreaded Bitcoin HD wallet address generator and seed manager written in Rust. Supports BIP44, BIP49, BIP84, and BIP86 derivation paths. Designed for research, auditing, and bulk key generation at massive scale — billions of addresses per hour on modern hardware.

---

## Show Support

I make nothing creating and sharing the tools I create. I do it for my love of the space and my love of the people in the space.

Help a fellow dev out, I aint vibe codinghere. Whats a sat or two between friends. :)

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
- Recalls addresses, seedwords and privkeys from seed index and address index

---

## Requirements

- Rust 1.76+
- PostgreSQL
- Linux (for best performance; WSL has known memory limits)

---

## Installation

```
cargo install bitcoin_mass_address_generator
```

---

## Configuration

Create a .env file in your root directory:

```
# PostgreSQL connection string
DATABASE_URL="postgres://user:pass@localhost:5432/wallet_addresses"

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

```
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

## After address creation

Add indexes to the database tables after creation.

```
CREATE INDEX ON addresses(seed_id, id);
CREATE INDEX ON keys(id);
```

---

## Usage

### Generate Addresses

```
./bitcoin_mass_address_generator generate
```

This will spawn N threads (defined in .env) to generate seed phrases and addresses, writing in bulk to PostgreSQL.

---

### Recall Wallet Data

```
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
