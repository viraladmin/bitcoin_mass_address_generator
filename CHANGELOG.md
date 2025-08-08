## [0.1.4] - 2025-08-08
### Added
- `examine <seed id>`: Checks if any of your addresses have a balance on bitcoin

### Uupdated
- Updated the .env file to access second database connect to the bitcoin balance ETL
- Updated the help command list for new features
- Updated the README.me file to reflect new command options

### Notes
- `examine` Requires [Bitcoin Balance ETL](https://github.com/viraladmin/bitcoin_balance_etl)
- if Bitcoin Balance ETL is not installed use an empty string in the .env


## [0.1.3] - 2025-08-07
### Added
- `write_file addresses <file> <limit>`: Export addresses to file.
- `write_file seeds <file> <limit>`: Export human-readable seed phrases.
- `write_file seeds_addresses <file> <limit>`: Export both seed + address (joined).
- updated help for new command options

### Notes
- Works seamlessly with the [solo Bitcoin miner](https://github.com/viraladmin/bitcoin_solo_miner)
- Helps automate never reusing an address per mined block
