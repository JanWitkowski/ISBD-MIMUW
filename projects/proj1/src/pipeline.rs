use std::env;
use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

mod hash;
use hash::stretch_md5;
use hash::sum_block_hash;

const BLOCK_SIZE: u64 = 1024 * 1024;
const K: u64 = 1000;
const NUM_READERS: usize = 1;
const NUM_COMPUTERS: usize = 1;
const CHANNEL_CAPACITY: usize = 2;

fn run_pipeline(
    file: &File,
    total_size: u64,
    block_size: u64,
    k: u64,
    num_readers: usize,
    num_computers: usize,
) -> io::Result<Vec<u8>> {
    let num_blocks = total_size.div_ceil(block_size);

    thread::scope(|scope| {
        let (sender, receiver) = sync_channel::<Vec<u8>>(CHANNEL_CAPACITY);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut reader_handles = Vec::with_capacity(num_readers);
        let mut compute_handles = Vec::with_capacity(num_computers);

        for reader_index in 0..num_readers {
            let sender = sender.clone();
            let range_start = num_blocks * reader_index as u64 / num_readers as u64;
            let range_end = num_blocks * (reader_index + 1) as u64 / num_readers as u64;

            reader_handles.push(scope.spawn(move || -> io::Result<()> {
                for block_index in range_start..range_end {
                    let offset = block_index * block_size;
                    let bytes_to_read = block_size.min(total_size - offset) as usize;
                    let mut buffer = vec![0u8; bytes_to_read];

                    file.read_exact_at(&mut buffer, offset)?;

                    if sender.send(buffer).is_err() {
                        return Ok(());
                    }
                }

                Ok(())
            }));
        }
        drop(sender);

        for _ in 0..num_computers {
            let receiver = Arc::clone(&receiver);

            compute_handles.push(scope.spawn(move || {
                let mut local_result = vec![0u8; 16];

                loop {
                    let received = receiver.lock().expect("channel lock poisoned").recv();
                    let buffer = match received {
                        Ok(buffer) => buffer,
                        Err(_) => break,
                    };

                    let block_hash = stretch_md5(&buffer, k);
                    sum_block_hash(&mut local_result, &block_hash);
                }

                local_result
            }));
        }

        let mut read_error = None;
        for reader in reader_handles {
            if let Err(error) = reader.join().expect("reader thread panicked") {
                read_error.get_or_insert(error);
            }
        }

        let mut result = vec![0u8; 16];
        for compute in compute_handles {
            let local_result = compute.join().expect("compute thread panicked");
            for index in 0..result.len() {
                result[index] ^= local_result[index];
            }
        }

        match read_error {
            Some(error) => Err(error),
            None => Ok(result),
        }
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!(
            "Usage: cargo run --release --bin pipeline filepath block_size K num_readers num_computers"
        );
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
    let k = args[3].parse::<u64>().unwrap_or(K);
    let num_readers = args[4].parse::<usize>().unwrap_or(NUM_READERS);
    let num_computers = args[5].parse::<usize>().unwrap_or(NUM_COMPUTERS);
    if num_readers == 0 || num_computers == 0 {
        eprintln!("num_readers and num_computers must be greater than zero");
        return;
    }
    let total_size = metadata.unwrap().len();
    let start = Instant::now();

    let result = match run_pipeline(&file, total_size, block_size, k, num_readers, num_computers) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Error reading file: {error}");
            return;
        }
    };

    let elapsed = start.elapsed();
    println!(
        "Pipeline | {} µs | M={} B | K={} | readers={} | computers={}",
        elapsed.as_micros(),
        block_size,
        k,
        num_readers,
        num_computers
    );

    print!("{} 0x", args[1]);
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn pipeline_hashes_every_block() {
        let data: Vec<u8> = (0..10_003).map(|value| (value % 251) as u8).collect();
        let path = env::temp_dir().join(format!("pipeline-test-{}.bin", std::process::id()));
        fs::write(&path, &data).unwrap();

        let block_size = 127;
        let k = 7;
        let mut expected = vec![0u8; 16];
        for block in data.chunks(block_size) {
            sum_block_hash(&mut expected, &stretch_md5(block, k));
        }

        let file = File::open(&path).unwrap();
        let actual = run_pipeline(&file, data.len() as u64, block_size as u64, k, 3, 4).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(actual, expected);
    }
}
