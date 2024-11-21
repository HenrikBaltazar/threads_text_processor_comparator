use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <file_path> <num_threads> <word>", args[0]);
        return Ok(());
    }

    let file_path = &args[1];
    let num_threads: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Invalid number of threads: {}", args[2]);
        std::process::exit(1);
    });
    let word_to_search = &args[3];
    
    let start = Instant::now();

    
    match count_word(file_path.to_string(), num_threads, word_to_search.to_string()) {
        Ok(count) => println!("The word '{}' appeared {} times.", word_to_search, count),
        Err(e) => eprintln!("Error: {}", e),
    }

    let duration = start.elapsed();
    println!("Time taken to process the file: {:?}", duration);

    Ok(())
}

fn count_word(file_path: String, num_threads: usize, word: String) -> io::Result<usize> {
    let file = File::open(&file_path)?;
    let file_size = file.metadata()?.len() as usize;
    let chunk_size = file_size / num_threads;

    let word_count = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for i in 0..num_threads {
        let file_path = file_path.clone();
        let word = word.clone();
        let word_count = Arc::clone(&word_count);

        let start = i * chunk_size;
        let end = if i == num_threads - 1 {
            file_size
        } else {
            start + chunk_size
        };

        let handle = thread::spawn(move || {
            let mut file = File::open(&file_path).expect("Failed to open file");
            let mut buffer = vec![0; end - start];
            file.seek(SeekFrom::Start(start as u64)).expect("Failed to seek");
            file.read_exact(&mut buffer).expect("Failed to read");

            let chunk_word_count = buffer.windows(word.len())
                .filter(|window| window == &word.as_bytes())
                .count();

            let mut word_count = word_count.lock().expect("Failed to lock counter");
            *word_count += chunk_word_count;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let word_count = word_count.lock().expect("Failed to lock counter");
    Ok(*word_count)
}
