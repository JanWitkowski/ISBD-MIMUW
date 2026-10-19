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
            elements: (0 as u64..size).collect(),
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
}

fn calculate_hash(
    file: &File,
    total_size: u64,
    offsets: &[u64],
    num_readers: usize,
    block_size: u64,
) -> io::Result<Vec<u8>> {
    let block_size_usize = usize::try_from(block_size)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "block size is too large"))?;
    let blocks_per_batch = (BATCH_TARGET_BYTES / block_size_usize).clamp(1, MAX_BLOCKS_PER_BATCH);
    let worker_count = num_readers.min(offsets.len().max(1));

    thread::scope(|scope| -> io::Result<Vec<u8>> {
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

                    for (slot, &offset) in batch_offsets.iter().enumerate() {
                        let bytes_to_read = block_size.min(total_size - offset) as usize;
                        let slot_start = slot * block_size_usize;
                        let target = &mut data[slot_start..slot_start + bytes_to_read];

                        if let Err(error) = file.read_exact_at(target, offset) {
                            let _ = tx.send(Err(error));
                            return;
                        }
                        block_lengths.push(bytes_to_read);
                    }

                    if tx
                        .send(Ok(ReadBatch {
                            data,
                            block_lengths,
                        }))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }

        drop(tx);

        let mut result = vec![0u8; 16];
        for read_result in rx {
            let batch = read_result?;
            for (slot, &block_len) in batch.block_lengths.iter().enumerate() {
                let slot_start = slot * block_size_usize;
                let block_hash = md5::compute(&batch.data[slot_start..slot_start + block_len]);
                sum_block_hash(&mut result, &block_hash);
            }
        }

        Ok(result)
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run --release --bin latency filepath num_readers block_size");
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
    let num_readers = args[2].parse::<usize>().unwrap_or(NUM_READERS);
    let block_size = args[3].parse::<u64>().unwrap_or(BLOCK_SIZE);
    if num_readers == 0 || block_size == 0 {
        eprintln!("num_readers and block_size must be greater than zero");
        return;
    }

    let total_size = metadata.unwrap().len();
    let num_blocks = (total_size + block_size - 1) / block_size;

    let now = Instant::now();

    let offsets: Vec<u64> = RandomOrderIterator::with_seed(num_blocks, RAND_SEED)
        .map(|block_idx| block_idx * block_size)
        .collect();

    let result = match calculate_hash(&file, total_size, &offsets, num_readers, block_size) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Error reading file: {error}");
            return;
        }
    };

    let elapsed = now.elapsed();
    println!(
        "Latency | {} µs | {} threads | {}",
        elapsed.as_micros(),
        num_readers,
        block_size
    );

    std::hint::black_box(result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parallel_readers_produce_the_expected_hash() {
        let data: Vec<u8> = (0..10_003).map(|value| (value % 251) as u8).collect();
        let path = env::temp_dir().join(format!("latency-test-{}.bin", std::process::id()));
        fs::write(&path, &data).unwrap();

        let block_size = 127;
        let num_blocks = data.len().div_ceil(block_size);
        let offsets: Vec<u64> = RandomOrderIterator::with_seed(num_blocks as u64, RAND_SEED)
            .map(|block_index| block_index * block_size as u64)
            .collect();

        let mut expected = vec![0u8; 16];
        for &offset in &offsets {
            let start = offset as usize;
            let end = (start + block_size).min(data.len());
            sum_block_hash(&mut expected, &md5::compute(&data[start..end]));
        }

        let file = File::open(&path).unwrap();
        let actual =
            calculate_hash(&file, data.len() as u64, &offsets, 4, block_size as u64).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(actual, expected);
    }
}
