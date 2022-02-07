use std::fs::File;
use std::fs::OpenOptions;
use std::error::Error;
use std::io::{self, prelude::*, BufReader};
use std::collections::HashSet;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use csv::ReaderBuilder;

type Word = [u8; 5];

#[derive(Debug)]
struct Constraints {
    known: HashMap<usize, u8>,
    known_not: HashMap<usize, u8>,
    included: HashSet<u8>,
    excluded: HashSet<u8>
}

fn load_words() -> Result<Vec<Word>, Box<dyn Error>> {
    let mut word_vec = Vec::new();

    let file = File::open("words-5.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let word: Word = line?.as_bytes().try_into().expect("word to fit into 5 bytes");
        word_vec.push(word);
    }
    Ok(word_vec)
}

fn make_constraints(guess: &Word, actual: &Word) -> Constraints {
    let mut known = HashMap::with_capacity(5);
    let mut known_not = HashMap::with_capacity(5);
    let mut included = HashSet::new();
    let mut excluded = HashSet::new();

    for i in 0..guess.len() {
        if guess[i] == actual[i] {
            known.insert(i, guess[i]);
        }
    }
    for i in 0..guess.len() {
        if !known.contains_key(&i) {
            let mut found = false;
            for j in 0..actual.len() {
                if guess[i] == actual[j] {
                    included.insert(guess[i]);
                    known_not.insert(i, guess[i]);
                    found = true;
                    break;
                }
            }
            if !found {
                excluded.insert(guess[i]);
            }
        }
    }
    Constraints {
        known: known,
        known_not: known_not,
        included: included,
        excluded: excluded
    }
}

fn check_constraints(word: &Word, constraints: &Constraints) -> bool {
    for (i, character) in constraints.known.iter() {
        if word[*i] != *character {
            return false;
        }
    }
    for (i, character) in constraints.known_not.iter() {
        if word[*i] == *character {
            return false;
        }
    }
    'included_chars: for character in constraints.included.iter() {
        for i in 0..word.len() {
            if word[i] == *character {
                continue 'included_chars;
            }
        }
        return false;
    }
    for i in 0..word.len() {
        if constraints.excluded.contains(&word[i]) {
            return false;
        }
    }
    true
}


fn init_output_file(filename: &str, words: &Vec<Word>) -> Result<(usize, File), Box<dyn Error>> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(filename)?;

    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(&file);

    let mut i = 0;
    for (word, record_result) in words.iter().zip(reader.records()) {
        let record = record_result?;
        let file_word: Word = record.get(1).ok_or("no wordle-word")?.as_bytes().try_into().expect("word to fit into 5 bytes");
        assert_eq!(word, &file_word, "input- and output-file not compatible");
        i += 1;
    }

    Ok((i, file))
}

fn main() -> io::Result<()> {
    let words = load_words().unwrap();
    let (start, mut output_file) = init_output_file("words-5-output.txt", &words).unwrap();

    println!("Starting at i={}", start);

    let mut scores = vec!(0.; words.len());

    let pb = ProgressBar::new(words.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta_precise})")
        .progress_chars("#>-"));
    pb.set_position(start as u64);

    for i in start..words.len() {
        for j in 0..words.len() {
            let guess = words[i];
            let actual = words[j];
            let constraints = make_constraints(&guess, &actual);

            for k in 0..words.len() {
                if check_constraints(&words[k], &constraints) {
                    scores[i] += 1.
                }
            }
        }
        scores[i] /= words.len() as f32;
        let word_string = String::from_utf8(words[i].to_vec()).unwrap();
        let line = format!("{},{}", scores[i], word_string);
        writeln!(output_file, "{}", line)?;
        pb.inc(1);
    }

    pb.finish();
    println!("Done!");
    Ok(())
}
