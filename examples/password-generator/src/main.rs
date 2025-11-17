use clap::Parser;

#[derive(Parser)]
#[command(about = "Generate secure passwords using quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "16")]
    length: usize,

    #[arg(short, long, default_value = "1")]
    count: usize,

    #[arg(long)]
    no_uppercase: bool,

    #[arg(long)]
    no_lowercase: bool,

    #[arg(long)]
    no_digits: bool,

    #[arg(long)]
    no_symbols: bool,

    #[arg(long)]
    passphrase: bool,
}

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

const WORDS: &[&str] = &[
    "correct", "horse", "battery", "staple", "dragon", "monkey", "treasure", "mountain",
    "river", "ocean", "forest", "desert", "island", "valley", "canyon", "meadow",
    "thunder", "lightning", "rainbow", "sunrise", "sunset", "winter", "summer", "spring",
    "apple", "orange", "banana", "grape", "cherry", "mango", "peach", "plum",
];

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        if args.passphrase {
            let passphrase = generate_passphrase(&args.gateway_url, &args.api_key, args.length);
            println!("{}", passphrase);
        } else {
            let password = generate_password(&args);
            println!("{}", password);
        }
    }
}

fn generate_password(args: &Args) -> String {
    let mut charset = String::new();
    
    if !args.no_uppercase { charset.push_str(UPPERCASE); }
    if !args.no_lowercase { charset.push_str(LOWERCASE); }
    if !args.no_digits { charset.push_str(DIGITS); }
    if !args.no_symbols { charset.push_str(SYMBOLS); }
    
    if charset.is_empty() {
        eprintln!("Error: At least one character set must be enabled");
        std::process::exit(1);
    }
    
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    let charset_len = charset_bytes.len();
    
    let random_bytes = get_random_bytes(&args.gateway_url, &args.api_key, args.length);
    
    random_bytes.iter()
        .map(|&b| charset_bytes[b as usize % charset_len] as char)
        .collect()
}

fn generate_passphrase(gateway_url: &str, api_key: &str, word_count: usize) -> String {
    let random_bytes = get_random_bytes(gateway_url, api_key, word_count);
    
    random_bytes.iter()
        .map(|&b| WORDS[b as usize % WORDS.len()])
        .collect::<Vec<_>>()
        .join("-")
}

fn get_random_bytes(gateway_url: &str, api_key: &str, count: usize) -> Vec<u8> {
    let url = format!(
        "{}/api/random?bytes={}&encoding=hex&api_key={}",
        gateway_url, count, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    hex::decode(hex_data.trim()).expect("Invalid hex data")
}
