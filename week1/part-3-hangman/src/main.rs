// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::collections::HashMap;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn find_element(vec: &Vec<char>, elem: char, map: &mut HashMap<char, i32>) -> Option<usize> {
    match map.get(&elem) {
        Some(appear_count) => {
            let mut count = 0;
            for (index, letter) in vec.iter().enumerate() {
                if letter == &elem {
                    count += 1;
                    if count == *appear_count {
                        *map.get_mut(&elem).unwrap() -= 1;
                        return Some(index);
                    }
                }
            }
        },
        None => {
            return None;
        }
    }
    return None;
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    let mut map: HashMap<char, i32> = HashMap::new(); // Record the times of a letter appears in the word
    let mut guess_word: Vec<char> = vec!['-'; secret_word_chars.len()];
    let mut guesses_count: u32 = 0;
    let mut guessed_letter = String::new();

    for letter in secret_word_chars.iter() {
        *map.entry(*letter).or_insert(0) += 1;
    }

    while guesses_count < NUM_INCORRECT_GUESSES {
        let mut guess = String::new();
        println!("The word so far is {}", guess_word.iter().collect::<String>());
        println!("You have guessed the following letters: {}", guessed_letter);
        println!("You have {} guesses left", NUM_INCORRECT_GUESSES - guesses_count);
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        io::stdin().read_line(&mut guess).expect("Error reading line.");
        if let Some(index) = guessed_letter.find(&guess.trim()) {
        }else {
            guessed_letter.push_str(guess.trim());
        }
        if let Some(position) = find_element(&secret_word_chars, guess.trim().chars().next().unwrap(), &mut map) {
            guess_word[position] = guess.trim().chars().next().unwrap();
        }else {
            guesses_count += 1;
            println!("Sorry, that letter is not in the word");
        }
        if guess_word.iter().collect::<String>() == secret_word {
            println!("Congratulations you guessed the word: {}", secret_word);
            break;
        }
    }
    if guess_word.iter().collect::<String>() != secret_word {
        println!("Sorry, you ran out of guesses!");
    }
}
