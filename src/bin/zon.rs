//! ZON Command Line Interface
//!
//! A CLI tool for converting files between JSON and ZON format.
//!
//! ## Usage
//!
//! ```bash
//! # Encode JSON to ZON format
//! zon encode data.json > data.zonf
//!
//! # Decode ZON back to JSON
//! zon decode data.zonf > output.json
//! ```

use std::env;
use std::fs;
use std::process;

use zon_format::{decode, encode};

fn print_usage() {
    eprintln!("Usage: zon <encode|decode> <file>");
    eprintln!("Example: zon encode data.json > data.zonf");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    let input_file = &args[2];

    // Read input file
    let content = match fs::read_to_string(input_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_file, e);
            process::exit(1);
        }
    };

    match command.as_str() {
        "encode" => {
            // Parse JSON
            let json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                    process::exit(1);
                }
            };

            // Encode to ZON
            match encode(&json) {
                Ok(encoded) => {
                    println!("{}", encoded);
                }
                Err(e) => {
                    eprintln!("Error encoding to ZON: {}", e);
                    process::exit(1);
                }
            }
        }
        "decode" => {
            // Decode ZON
            match decode(&content) {
                Ok(decoded) => {
                    // Pretty print JSON
                    match serde_json::to_string_pretty(&decoded) {
                        Ok(json_str) => {
                            println!("{}", json_str);
                        }
                        Err(e) => {
                            eprintln!("Error serializing to JSON: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error decoding ZON: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}
