use once_cell::sync::Lazy;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::io::BufRead;
use std::sync::Arc;

thread_local! {
    pub static THREAD_RNG: RefCell<ChaCha20Rng> = RefCell::new(ChaCha20Rng::from_entropy());
}

// Static global wordlist
pub static WORDLIST: Lazy<Arc<Vec<String>>> = Lazy::new(|| {
    let file = std::fs::File::open("wordlist.txt").expect("wordlist.txt missing");
    let reader = std::io::BufReader::new(file);
    Arc::new(reader.lines().collect::<Result<_, _>>().expect("Invalid wordlist"))
});
