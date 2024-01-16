use rand::seq::SliceRandom;
use rand::thread_rng;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

#[derive(Debug, Copy, Clone)]
enum Match {
    FULL, // letter exists and in correct index
    HALF, // letter exists but in other index
    NONE, // letter don't exists
}

fn load_words(content: String) -> Vec<String> {
    content
        .split("\n")
        .filter(|w| w.len() == 5 && w.chars().all(char::is_alphabetic))
        .map(|w| w.to_uppercase())
        .collect()
}

fn input_guess(attempt_no: u8) -> Result<String, Box<dyn Error>> {
    loop {
        let mut input = String::new();

        print!("{} ", attempt_no);
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_uppercase();

        if !input.chars().all(char::is_alphabetic) {
            println!("INFO: Word should contain only alphabets");
        } else if input.len() != 5 {
            println!("INFO: Must provide word of length 5");
        } else {
            return Ok(input.to_uppercase());
        }
    }
}

fn random_word<'a>(words: &'a Vec<String>) -> Option<&'a String> {
    words.choose(&mut thread_rng())
}

fn evaluate_guess(guess: &String, chosen_word: Option<&String>) -> Option<([Match; 5], u8)> {
    let chosen = chosen_word.as_ref()?;
    let mut matches = [Match::NONE; 5];
    let mut full_match_count: u8 = 0;

    for (i, (guess_ch, chosen_ch)) in guess.chars().zip(chosen.chars()).enumerate() {
        if guess_ch == chosen_ch {
            matches[i] = Match::FULL;
            full_match_count += 1;
        } else if chosen.contains(guess_ch) {
            matches[i] = Match::HALF;
        } else {
            matches[i] = Match::NONE;
        }
    }

    Some((matches, full_match_count))
}

fn format_match(guess: &String, match_result: [Match; 5]) -> String {
    let mut segments = Vec::new();

    for (mtype, ch) in match_result.iter().zip(guess.chars()) {
        segments.push(match *mtype {
            Match::FULL => String::from("\x1b[30;42m"),
            Match::HALF => String::from("\x1b[30;43m"),
            Match::NONE => String::from("\x1b[30;47m"),
        });
        segments.push(String::from(ch));
    }

    segments.push(String::from("\x1b[0m"));
    segments.join(" ")
}

fn playagain() -> Result<bool, Box<dyn Error>> {
    loop {
        let mut input = String::new();
        println!("Playagain? [y/N]");
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input == "y" {
            return Ok(true);
        } else if input == "N" {
            return Ok(false);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: program [wordsfile]");
        process::exit(1);
    }

    let source_file = if args.len() == 2 {
        args[1].clone()
    } else {
        String::from("data/words5.txt")
    };

    let words = match fs::read_to_string(&source_file) {
        Ok(content) => load_words(content),
        Err(e) => {
            eprintln!("Error occured while reading file: {} \n{}", source_file, e);
            process::exit(1);
        }
    };

    if words.len() == 0 {
        eprintln!("No appropriate words found in file: {}", source_file);
        process::exit(1);
    }

    let wordle = "\x1b[30;41m W \x1b[30;42m O \x1b[30;43m R \x1b[30;44m D \x1b[30;45m L \x1b[30;46m E \x1b[0m";

    let max_attempts: u8 = 6;
    let mut attempts: u8 = 0;
    let mut chosen_word: Option<&String> = None;

    loop {
        if let None = chosen_word {
            chosen_word = random_word(&words);
            attempts = 0;
            println!("\n{}\n", wordle);
        }

        attempts += 1;

        let guess = input_guess(attempts).unwrap_or_else(|err| {
            eprintln!("Error while tking input: {}", err);
            process::exit(1);
        });

        let (matches, match_count) = match evaluate_guess(&guess, chosen_word) {
            Some(res) => res,
            None => {
                eprintln!("Something went wrong, guessed word is None!");
                process::exit(1);
            }
        };

        println!("{}\n", format_match(&guess, matches));

        if match_count == 5 {
            println!("You WON!");
        } else if attempts >= max_attempts {
            println!("You LOST!");
            match chosen_word {
                Some(word) => println!("Word: {}", word),
                None => {
                    eprintln!("Something went wrong: No word is chosen!");
                    process::exit(1);
                }
            };
        } else {
            continue;
        }

        let keep_playing = playagain().unwrap_or_else(|err| {
            eprintln!("Error occured while reading input: {}", err);
            process::exit(1);
        });

        if keep_playing {
            chosen_word = None;
        } else {
            break;
        }
    }
}
