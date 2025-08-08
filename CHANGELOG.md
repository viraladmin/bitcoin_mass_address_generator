## [0.1.3] - 2025-08-07
### Added
- `write_file addresses <file> <limit>`: Export addresses to file.
- `write_file seeds <file> <limit>`: Export human-readable seed phrases.
- `write_file seeds_addresses <file> <limit>`: Export both seed + address (joined).
- updated help for new command options

### Notes
- Works seamlessly with the [solo Bitcoin miner](https://github.com/viraladmin/bitcoin_solo_miner)
- Helps automate never reusing an address per mined block
