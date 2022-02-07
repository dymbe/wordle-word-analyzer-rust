use std::fs::{File, OpenOptions};
use std::error::Error;
use std::io::{self, prelude::*, BufReader};
use std::collections::HashSet;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use indicatif::ParallelProgressIterator;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

type Word = [u8; 5];

#[derive(Debug)]
struct Constraints {
    known: HashMap<usize, u8>,
    known_not: HashMap<usize, u8>,
    included: HashSet<u8>,
    excluded: HashSet<u8>
}

fn read_words() -> Result<Vec<Word>, Box<dyn Error>> {
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

fn main() -> io::Result<()> {
    let words = read_words().unwrap();

    let pb = ProgressBar::new(words.len() as u64)
        .with_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta_precise})")
        .progress_chars("#>-"));

    let scores: Vec<f32> = words.par_iter().progress_with(pb)
        .map(|guess| {
            let mut score = 0.;
            for actual in words.iter() {
                let constraints = make_constraints(&guess, &actual);
                for test_word in words.iter() {
                    if check_constraints(&test_word, &constraints) {
                        score += 1.
                    }
                }
            }
            score / words.len() as f32
        })
        .collect();

    let mut scores_and_words: Vec<(&f32, &Word)> = scores.iter().zip(words.iter())
        .collect::<Vec<(&f32, &Word)>>();
    scores_and_words.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let content: String = scores_and_words.iter()
        .map(|(score, word_bytes)| {
            let word_string = String::from_utf8(word_bytes.to_vec()).unwrap();
            format!("{},{}", score, word_string)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("words-5-output.txt")?;

    write!(file, "{}", content)?;
    println!("Done!");

    Ok(())
}
