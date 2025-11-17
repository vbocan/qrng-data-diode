use clap::Parser;

#[derive(Parser)]
#[command(about = "Roll dice using quantum random numbers")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(help = "Dice notation (e.g., 3d6, 2d20+5, d100)")]
    dice: String,
}

fn main() {
    let args = Args::parse();
    
    let (num_dice, sides, modifier) = parse_dice(&args.dice);
    
    let mut rolls = Vec::new();
    let mut total = 0;
    
    for _ in 0..num_dice {
        let roll = get_random_int(&args.gateway_url, &args.api_key, 1, sides + 1);
        rolls.push(roll);
        total += roll;
    }
    
    total += modifier;
    
    println!("Rolling {}", args.dice);
    println!("Rolls: {:?}", rolls);
    if modifier != 0 {
        println!("Modifier: {:+}", modifier);
    }
    println!("Total: {}", total);
}

fn parse_dice(notation: &str) -> (usize, i64, i64) {
    let notation = notation.to_lowercase();
    
    let (dice_part, modifier) = if notation.contains('+') {
        let parts: Vec<&str> = notation.split('+').collect();
        (parts[0], parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0))
    } else if notation.contains('-') {
        let parts: Vec<&str> = notation.split('-').collect();
        (parts[0], -parts.get(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0))
    } else {
        (notation.as_str(), 0)
    };
    
    let parts: Vec<&str> = dice_part.split('d').collect();
    let num_dice = if parts[0].is_empty() { 
        1 
    } else { 
        parts[0].parse().unwrap_or(1) 
    };
    let sides = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    
    (num_dice, sides, modifier)
}

fn get_random_int(gateway_url: &str, api_key: &str, min: i64, max: i64) -> i64 {
    let range = (max - min) as u64;
    
    let url = format!(
        "{}/api/random?bytes=8&encoding=hex&api_key={}",
        gateway_url, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
    
    let mut array = [0u8; 8];
    array.copy_from_slice(&bytes[0..8]);
    let random_u64 = u64::from_le_bytes(array);
    
    min + (random_u64 % range) as i64
}
