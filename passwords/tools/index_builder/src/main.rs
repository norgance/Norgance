use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

fn main() {
    let input_file_path = std::env::args().nth(1).unwrap_or(String::from("input.csv"));
    let index_file_path = std::env::args().nth(2).unwrap_or(String::from("index.csv"));

    let input_file = File::open(input_file_path).expect("Unable to open file");
    let output_file = File::create(index_file_path).expect("Unable to open index file");
    let input_buffer = BufReader::new(input_file);
    let mut output_buffer = BufWriter::new(output_file);

    let mut current_line_number = 0u64;
    let mut from_line_number = 0u64;
    let mut current_hash_prefix = String::from("");
    for line in input_buffer.lines() {
        let unwrapped_line = line.expect("Unreadable line");
        let hash_prefix: String = unwrapped_line.chars().take(5).collect();
        if hash_prefix != current_hash_prefix {
            if current_line_number >= 1 {
                register_hash(
                    &mut output_buffer,
                    current_hash_prefix,
                    from_line_number,
                    current_line_number - 1,
                );
            }
            current_hash_prefix = hash_prefix;
            from_line_number = current_line_number;
        }
        current_line_number += 1;
    }
    register_hash(
        &mut output_buffer,
        current_hash_prefix,
        from_line_number,
        current_line_number - 1,
    );
    output_buffer.flush().unwrap();
}

fn register_hash(output: &mut BufWriter<std::fs::File>, hash: String, from: u64, to: u64) {
    output
        .write(format!("{},{},{}\n", hash, from, to).as_bytes())
        .expect("Unable to write to output");
}
