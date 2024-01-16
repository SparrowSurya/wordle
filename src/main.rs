use rand::seq::SliceRandom;
use rand::thread_rng;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

const WORDS_FILE: &str = "data/words5.txt";

// Source for words
struct WordSource<'a> {
    file: &'a str,   // file from which words are read
    content: String, // content of the file
}

impl WordSource<'_> {
    fn load(file: &str) -> Result<WordSource, Box<dyn Error>> {
        let content = fs::read_to_string(file)?;
        Ok(WordSource { file, content })
    }
}

// state of current
struct State<'a> {
    chosen: Option<&'a String>, // wordto guess for (uppercase)
    attempts: u64,              // attempts made by user
    max_attempts: u64,          // maximum attempts allowed
}

impl<'a> State<'a> {
    fn init(max_attempts: u64) -> State<'static> {
        State {
            chosen: None,
            attempts: 0 as u64,
            max_attempts,
        }
    }

    fn reset(&mut self) {
        self.chosen = None;
        self.attempts = 0 as u64;
    }
}

// kind of match
#[derive(Debug, Copy, Clone)]
enum Match {
    FULL, // letter exists and in correct index
    HALF, // letter exists but in other index
    NONE, // letter don't exists
}

fn read_words<'a>(source: &'a WordSource) -> Vec<String> {
    source
        .content
        .split("\n")
        .filter(|w| w.len() == 5 && w.chars().all(char::is_alphabetic))
        .map(|w| w.to_uppercase())
        .collect()
}

fn input_guess(attempt_no: u64) -> Result<String, Box<dyn Error>> {
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

fn evaluate_guess(state: &mut State, guess: &String) -> Option<([Match; 5], u8)> {
    let chosen = state.chosen.as_ref()?;
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

    state.attempts += 1;

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
    let source = WordSource::load(WORDS_FILE).unwrap_or_else(|err| {
        eprintln!("Error occured while reading file: {} \n{}", WORDS_FILE, err);
        process::exit(1);
    });

    let words: Vec<String> = read_words(&source);
    let mut state = State::init(6);

    println!("\n\x1b[30;41m W \x1b[30;42m O \x1b[30;43m R \x1b[30;44m D \x1b[30;45m L \x1b[30;46m E \x1b[0m \n");

    loop {
        if let None = state.chosen {
            state.chosen = random_word(&words);
            state.attempts = 0;
        }

        let guess = input_guess(state.attempts + 1).unwrap_or_else(|err| {
            eprintln!("Error while tking input: {}", err);
            process::exit(1);
        });

        let (matches, match_count) = match evaluate_guess(&mut state, &guess) {
            Some(res) => res,
            None => {
                eprintln!("Something went wrong, guessed word is None!");
                process::exit(1);
            }
        };

        println!("{}\n", format_match(&guess, matches));

        if match_count == 5 {
            println!("You WON!");
        } else if state.attempts >= state.max_attempts {
            println!("You LOST!");
            if let Some(word) = state.chosen {
                println!("Word: {}", word.to_uppercase());
            }
        } else {
            continue;
        }

        let keep_playing = playagain().unwrap_or_else(|err| {
            eprintln!("Error occured while reading input: {}", err);
            process::exit(1);
        });

        if keep_playing {
            state.reset();
            println!("\n\x1b[30;41m W \x1b[30;42m O \x1b[30;43m R \x1b[30;44m D \x1b[30;45m L \x1b[30;46m E \x1b[0m \n");
        } else {
            break;
        }
    }
}
