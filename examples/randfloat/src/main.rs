use clap::Parser;

#[derive(Parser)]
#[command(about = "Generate random floating-point numbers using quantum entropy")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "1")]
    count: usize,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        let url = format!(
            "{}/api/random?bytes=8&encoding=hex&api_key={}",
            args.gateway_url, args.api_key
        );

        let response = reqwest::blocking::get(&url)
            .expect("Failed to contact gateway");
        
        let hex_data = response.text().expect("Failed to read response");
        let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
        
        let mut array = [0u8; 8];
        array.copy_from_slice(&bytes[0..8]);
        let random_u64 = u64::from_le_bytes(array);
        
        let random_f64 = (random_u64 as f64) / (u64::MAX as f64);
        
        println!("{}", random_f64);
    }
}
