extern crate sha1;
extern crate hex;

use std::io::{self, BufReader, BufWriter};
use std::io::prelude::*;
use std::fs::File;
use blake2_rfc::blake2b::{blake2b};

fn main() -> io::Result<()> {
    let input_file_path = std::env::args().nth(1).unwrap_or(String::from("input.txt"));
    let output_file_path = std::env::args().nth(2).unwrap_or(String::from("output.csv"));

    let input_file = File::open(input_file_path)?;
    let output_file = File::create(output_file_path)?;
    let input_buffer = BufReader::new(input_file);
    let mut output_buffer = BufWriter::new(output_file);

    for line in input_buffer.lines() {
        let unwrapped_line = line?;
        let mut fields = unwrapped_line.split(':');
        let hash = fields.next().expect("We are missing a hash on a line");
        let norgance_hash = norgance_password_hash(hash);
        let count = fields.next().unwrap_or("0");
        let count_int = count.trim().parse::<u32>().expect("Unable to parse count");
        let norgance_count = norgance_password_count(count_int);
        output_buffer.write(format!("{},{}\n", norgance_hash, norgance_count).as_bytes()).expect("Unable to write to output");
    }

    output_buffer.flush().unwrap();
    Ok(())
}

fn norgance_password_hash(sha1: &str) -> String {
    // The first step is to compute a sha1 of the password
    // because our passwords datasets provides sha1 hashes
    let password_sha1 = hex::decode(sha1).unwrap();

    // The next step is about computing a blake2b of the password
    // with some salt specific to norgance. So someone intercepting
    // the request on the network would have to spend quite a lot
    // of ressources to find out what was the origin of the hash

    // The salt is norgance- followed by the first 32 digits of ln(3)
    // (prononced Hélène de Troie in French, for Helen of Troy)
    // https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number

    // We use 20 bytes to have the same size than the previous sha1 checksums
    let hash = blake2b(20, b"norgance-1.0986122886681096913952452369225", &password_sha1);
    return hex::encode(hash);
}

// The idea is to return a letter based on a logarithmic scale from 0 to 1 000 000
// instead on the normal count
fn norgance_password_count(count: u32) -> char {
    // =PLAFOND.MATH((LOG10(count)/LOG10(1000000)*16))
    // log10(1_000_000) => 6
    // = (8xLOG10(count))/3
    if count == 0 {
        return '0';
    }

    let code = ((8f32 * (count as f32).log10()) / 3f32).min(15f32).round() as u8;

    match code {
        0..=9 => return char::from(48 + code),
        10..=15 => return char::from(55 + code),
        _ => return '0',
    }
}
