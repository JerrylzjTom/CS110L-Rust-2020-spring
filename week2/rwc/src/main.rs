use std::{env, io};
use std::fs::File;
use std::io::BufRead;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)
    let mut nums_lines = 0;
    let mut nums_words = 0;
    let mut nums_chars = 0;
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(error) => {
            println!("Error opening file: {}", error);
            process::exit(1);
        }
    };
    for line in io::BufReader::new(file).lines() {
        let line_str = match line {
            Ok(line_str) => line_str,
            Err(error) => {
                println!("Error reading line: {}", error);
                process::exit(1);
            }
        };
        nums_lines += 1;
        nums_words += line_str.split_whitespace().count();
        nums_chars += line_str.chars().count();
    }
    println!("{} {} {}", nums_lines, nums_words, nums_chars);
}
