use rand::rngs::StdRng;
use rand::{rng, Rng, RngExt, SeedableRng};
use std::env;
use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

mod hash;
use hash::sum_block_hash;

pub struct RandomOrderIterator {
    elements: Vec<u64>,
    rng: Box<dyn Rng>,
}

impl RandomOrderIterator {
    pub fn with_seed(size: u64, seed: u64) -> Self {
        Self {
            elements: (0..size).collect(),
            rng: Box::new(StdRng::seed_from_u64(seed)),
        }
    }

    pub fn new(size: u64) -> Self {
        Self {
            elements: (0..size).collect(),
            rng: Box::new(rng()),
        }
    }
}

impl Iterator for RandomOrderIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements.is_empty() {
            None
        } else {
            let rand_idx = self.rng.random_range(0..self.elements.len());
            Some(self.elements.swap_remove(rand_idx))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.elements.len(), Some(self.elements.len()))
    }
}

const BLOCK_SIZE: u64 = 512;
const RAND_SEED: u64 = 1234;
const NUM_READERS: usize = 4;

const BATCH_TARGET_BYTES: usize = 256 * 1024;
const MAX_BLOCKS_PER_BATCH: usize = 1024;

struct ReadBatch {
    data: Vec<u8>,
    block_lengths: Vec<usize>,
    read_latencies_ns: Vec<u64>,
}

struct Measurement {
    hash: Vec<u8>,
    read_latencies_ns: Vec<u64>,
}

fn calculate_hash(
    file: &File,
    total_size: u64,
    offsets: &[u64],
    num_readers: usize,
    block_size: u64,
) -> io::Result<Measurement> {
    let block_size_usize = usize::try_from(block_size)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "block size is too large"))?;
    let blocks_per_batch = (BATCH_TARGET_BYTES / block_size_usize).clamp(1, MAX_BLOCKS_PER_BATCH);
    let worker_count = num_readers.min(offsets.len().max(1));

    thread::scope(|scope| -> io::Result<Measurement> {
        let (tx, rx) = sync_channel::<io::Result<ReadBatch>>(worker_count);

        for worker_index in 0..worker_count {
            let tx = tx.clone();
            let range_start = offsets.len() * worker_index / worker_count;
            let range_end = offsets.len() * (worker_index + 1) / worker_count;
            let assigned_offsets = &offsets[range_start..range_end];

            scope.spawn(move || {
                for batch_offsets in assigned_offsets.chunks(blocks_per_batch) {
                    let mut data = vec![0u8; batch_offsets.len() * block_size_usize];
                    let mut block_lengths = Vec::with_capacity(batch_offsets.len());
                    let mut read_latencies_ns = Vec::with_capacity(batch_offsets.len());

                    for (slot, &offset) in batch_offsets.iter().enumerate() {
                        let bytes_to_read = block_size.min(total_size - offset) as usize;
                        let slot_start = slot * block_size_usize;
                        let target = &mut data[slot_start..slot_start + bytes_to_read];

                        // Mierzymy wyłącznie czas synchronicznego pread.
                        let read_start = Instant::now();
                        let read_result = file.read_exact_at(target, offset);
                        let latency_ns =
                            read_start.elapsed().as_nanos().min(u64::MAX as u128) as u64;

                        if let Err(error) = read_result {
                            let _ = tx.send(Err(error));
                            return;
                        }

                        block_lengths.push(bytes_to_read);
                        read_latencies_ns.push(latency_ns);
                    }

                    if tx
                        .send(Ok(ReadBatch {
                            data,
                            block_lengths,
                            read_latencies_ns,
                        }))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }

        drop(tx);

        let mut hash = vec![0u8; 16];
        let mut read_latencies_ns = Vec::with_capacity(offsets.len());

        for read_result in rx {
            let batch = read_result?;
            read_latencies_ns.extend(batch.read_latencies_ns);

            for (slot, &block_len) in batch.block_lengths.iter().enumerate() {
                let slot_start = slot * block_size_usize;
                let block_hash = md5::compute(&batch.data[slot_start..slot_start + block_len]);
                sum_block_hash(&mut hash, &block_hash);
            }
        }

        Ok(Measurement {
            hash,
            read_latencies_ns,
        })
    })
}

fn print_average_read_latency(latencies_ns: &[u64], num_readers: usize, block_size: u64) {
    if latencies_ns.is_empty() {
        println!(
            "Latency2 | no reads | {} threads | {} B",
            num_readers, block_size
        );
        return;
    }

    let sum_ns: u128 = latencies_ns.iter().map(|&value| value as u128).sum();
    let average_us = sum_ns as f64 / latencies_ns.len() as f64 / 1_000.0;
    println!(
        "Latency2 | {:.3} µs avg read | {} threads | {} B",
        average_us, num_readers, block_size
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run --release --bin latency2 filepath num_readers block_size");
        return;
    }

    let file = match File::open(&args[1]) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error opening file {}: {error}", args[1]);
            return;
        }
    };

    let total_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            eprintln!("Error reading metadata for {}: {error}", args[1]);
            return;
        }
    };

    let num_readers = args[2].parse::<usize>().unwrap_or(NUM_READERS);
    let block_size = args[3].parse::<u64>().unwrap_or(BLOCK_SIZE);
    if num_readers == 0 || block_size == 0 {
        eprintln!("num_readers and block_size must be greater than zero");
        return;
    }

    let num_blocks = total_size.div_ceil(block_size);
    let offsets: Vec<u64> = RandomOrderIterator::with_seed(num_blocks, RAND_SEED)
        .map(|block_index| block_index * block_size)
        .collect();

    let measurement = match calculate_hash(&file, total_size, &offsets, num_readers, block_size) {
        Ok(measurement) => measurement,
        Err(error) => {
            eprintln!("Error reading file: {error}");
            return;
        }
    };

    print_average_read_latency(&measurement.read_latencies_ns, num_readers, block_size);

    std::hint::black_box(measurement.hash);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn measures_every_read_and_preserves_the_hash() {
        let data: Vec<u8> = (0..10_003).map(|value| (value % 251) as u8).collect();
        let path = env::temp_dir().join(format!("latency2-test-{}.bin", std::process::id()));
        fs::write(&path, &data).unwrap();

        let block_size = 127;
        let num_blocks = data.len().div_ceil(block_size);
        let offsets: Vec<u64> = RandomOrderIterator::with_seed(num_blocks as u64, RAND_SEED)
            .map(|block_index| block_index * block_size as u64)
            .collect();

        let mut expected_hash = vec![0u8; 16];
        for &offset in &offsets {
            let start = offset as usize;
            let end = (start + block_size).min(data.len());
            sum_block_hash(&mut expected_hash, &md5::compute(&data[start..end]));
        }

        let file = File::open(&path).unwrap();
        let measurement =
            calculate_hash(&file, data.len() as u64, &offsets, 4, block_size as u64).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(measurement.hash, expected_hash);
        assert_eq!(measurement.read_latencies_ns.len(), offsets.len());
    }
}
