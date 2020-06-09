extern crate rusqlite;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let input_file_path = std::env::args().nth(1).unwrap_or(String::from("input.csv"));
    let database_path = std::env::args()
        .nth(2)
        .unwrap_or(String::from("database.db"));

    let input_file = File::open(input_file_path).expect("Unable to open file");
    let mut database = rusqlite::Connection::open(database_path).expect("Unable to open database");

    database
        .execute(
            "CREATE TABLE IF NOT EXISTS hashes (
                  hash           TEXT PRIMARY KEY NOT NULL,
                  from_line      INTEGER NOT NULL,
                  to_line        INTEGER NOT NULL
                  ) WITHOUT ROWID",
            rusqlite::params![],
        )
        .expect("Unable to create the table hashes.");
    
    let transaction = database.transaction().expect("Unable to start transaction");

    let input_buffer = BufReader::new(input_file);
    let mut current_line_number = 0u64;
    let mut from_line_number = 0u64;
    let mut current_hash_prefix = String::from("");
    for line in input_buffer.lines() {
        let unwrapped_line = line.expect("Unreadable line");
        let hash_prefix: String = unwrapped_line.chars().take(5).collect();
        if hash_prefix != current_hash_prefix {
            if current_line_number >= 1 {
                register_hash(&transaction, current_hash_prefix, from_line_number, current_line_number - 1);
            }
            current_hash_prefix = hash_prefix;
            from_line_number = current_line_number;
        }
        current_line_number += 1;
    }
    register_hash(&transaction, current_hash_prefix, from_line_number, current_line_number - 1);
    transaction.commit().expect("Unable to commit");
}

fn register_hash(transaction: &rusqlite::Transaction, hash: String, from: u64, to: u64) {
    transaction.execute(
        "INSERT INTO hashes (hash, from_line, to_line) VALUES (?1, ?2, ?3)",
        rusqlite::params![hash, from as i64, to as i64],
    ).expect("Unable to insert hash into database");
}
