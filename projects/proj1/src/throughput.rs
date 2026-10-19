use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::time::Instant;

mod hash;
use hash::sum_block_hash;

const BLOCK_SIZE: u64 = 512;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --release --bin throughput filepath block_size");
        return;
    }

    let file_res = File::open(args[1].clone());
    if file_res.is_err() {
        eprintln!("File not found: {}", args[1]);
        return;
    }
    let file = file_res.unwrap();

    let metadata = file.metadata();
    if metadata.is_err() {
        eprintln!("Error opening file: {}", args[1]);
        return;
    }
    let block_size = args[2].parse::<u64>().unwrap_or(BLOCK_SIZE);
    let total_size = metadata.unwrap().len();
    let num_blocks = total_size.div_ceil(block_size);

    let mut buffer = vec![0u8; block_size as usize];
    let mut result = vec![0u8; 16];
    let start = Instant::now();

    for block_idx in 0..num_blocks {
        let offset = block_idx * block_size;
        let bytes_to_read = block_size.min(total_size - offset) as usize;

        // read
        let target_buf = &mut buffer[..bytes_to_read];
        let err = file.read_exact_at(target_buf, offset);
        if err.is_err() {
            eprintln!("Error reading file: {}", err.unwrap_err());
            return;
        }

        // compute
        let block_hash = md5::compute(target_buf);
        sum_block_hash(&mut result, &block_hash);
    }

    let elapsed = start.elapsed();
    println!(
        "Throughput | {} µs | M={} B",
        elapsed.as_micros(),
        block_size
    );

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!();
}
