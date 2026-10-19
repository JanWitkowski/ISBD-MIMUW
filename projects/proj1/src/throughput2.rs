use std::env;
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::ptr::NonNull;
use std::slice;
use std::time::Instant;

mod hash;
use hash::sum_block_hash;

const BLOCK_SIZE: u64 = 512;

struct MappedFile {
    pointer: NonNull<u8>,
    length: usize,
}

impl MappedFile {
    fn new(file: &File, length: usize) -> io::Result<Self> {
        if length == 0 {
            return Ok(Self {
                pointer: NonNull::dangling(),
                length,
            });
        }

        let pointer = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };

        if pointer == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            pointer: NonNull::new(pointer.cast()).unwrap(),
            length,
        })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.pointer.as_ptr(), self.length) }
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        if self.length != 0 {
            unsafe {
                libc::munmap(self.pointer.as_ptr().cast(), self.length);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --release --bin throughput2 filepath block_size");
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
    let total_size = metadata.unwrap().len() as usize;
    let mut result = vec![0u8; 16];
    let start = Instant::now();

    let mapped_file = match MappedFile::new(&file, total_size) {
        Ok(mapped_file) => mapped_file,
        Err(error) => {
            eprintln!("Error mapping file: {error}");
            return;
        }
    };

    for block in mapped_file.as_slice().chunks(block_size as usize) {
        let block_hash = md5::compute(block);
        sum_block_hash(&mut result, &block_hash);
    }

    let elapsed = start.elapsed();
    println!(
        "Throughput mmap | {} µs | M={} B",
        elapsed.as_micros(),
        block_size
    );

    print!("{} 0x", args[1]);
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!();
}
