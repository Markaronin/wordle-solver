use std::collections::{BTreeSet, HashMap};
use std::fs::*;
use std::io::{BufRead, BufReader};

fn read_file_lines(file_name: &str) -> Vec<String> {
    let file = File::open(file_name).expect("No such file");
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}

const ALL_CHARS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct WordlePosition {
    possible_characters_in_position: BTreeSet<char>,
}
impl WordlePosition {
    fn new() -> Self {
        WordlePosition {
            possible_characters_in_position: ALL_CHARS.clone().into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct WordleStatus {
    info: [WordlePosition; 5],
    must_contain: BTreeSet<char>,
}
impl WordleStatus {
    fn new() -> Self {
        WordleStatus {
            info: [
                WordlePosition::new(),
                WordlePosition::new(),
                WordlePosition::new(),
                WordlePosition::new(),
                WordlePosition::new(),
            ],
            must_contain: BTreeSet::new(),
        }
    }
    fn with_data(greens: [Option<char>; 5], yellows: [Vec<char>; 5], greys: Vec<char>) -> Self {
        let mut new_status = Self::new();
        for (possible_green, i) in greens.into_iter().zip(0..5) {
            if let Some(green) = possible_green {
                new_status.info[i].possible_characters_in_position = [green].into_iter().collect();
            }
        }
        for (yellows_in_position, i) in yellows.into_iter().zip(0..5) {
            for yellow in yellows_in_position {
                new_status.info[i]
                    .possible_characters_in_position
                    .remove(&yellow);
                new_status.must_contain.insert(yellow);
            }
        }
        for grey in greys.into_iter() {
            for i in 0..5 {
                new_status.info[i]
                    .possible_characters_in_position
                    .remove(&grey);
            }
        }
        new_status
    }

    fn guess_word(&self, guess: &Word, answer: &Word) -> Self {
        let mut new_status = self.clone();

        for i in 0..5 {
            if guess.chars[i] == answer.chars[i] {
                new_status.info[i].possible_characters_in_position =
                    [guess.chars[i]].into_iter().collect();
            } else if answer.chars.contains(&guess.chars[i]) {
                new_status.info[i]
                    .possible_characters_in_position
                    .remove(&guess.chars[i]);
                new_status.must_contain.insert(guess.chars[i]);
            } else {
                for wordle_position in &mut new_status.info {
                    wordle_position
                        .possible_characters_in_position
                        .remove(&guess.chars[i]);
                }
            }
        }

        new_status
    }

    fn word_matches(&self, word: &Word) -> bool {
        self.must_contain.iter().all(|c| word.chars.contains(c))
            && self
                .info
                .iter()
                .zip(word.chars)
                .all(|(wordle_position, c)| {
                    wordle_position.possible_characters_in_position.contains(&c)
                })
    }

    fn get_possible_answers(&self, allowed_answers: &Vec<Word>) -> Vec<Word> {
        allowed_answers
            .iter()
            .filter(|word| self.word_matches(word))
            .cloned()
            .collect()
    }

    fn best_guess(&self, allowed_guesses: &Vec<Word>, allowed_answers: &Vec<Word>) -> Word {
        let possible_answers = self.get_possible_answers(allowed_answers);
        if possible_answers.len() < 3 {
            return possible_answers.get(0).unwrap().clone();
        }
        let mut num_possible_answers_cache: HashMap<WordleStatus, usize> = HashMap::new();
        let (average, best_guess) = allowed_guesses
            .iter()
            .map(|guess| {
                let average_future_answers = possible_answers
                    .iter()
                    .map(|answer| {
                        let post_guess_status = self.guess_word(guess, answer);
                        num_possible_answers_cache
                            .entry(post_guess_status.clone())
                            .or_insert_with(|| {
                                post_guess_status
                                    .get_possible_answers(&possible_answers)
                                    .len()
                            })
                            .clone()
                            - 1
                    })
                    .sum::<usize>() as f64
                    / possible_answers.len() as f64;
                println!("{}: {average_future_answers}", guess.to_string());
                (average_future_answers, guess)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        println!("{average}");
        best_guess.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Word {
    chars: [char; 5],
}
impl Word {
    fn new(string: &str) -> Self {
        let mut chars = string.chars();
        Word {
            chars: [
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
            ],
        }
    }
    fn to_string(&self) -> String {
        self.chars.iter().collect()
    }
}

fn main() {
    let allowed_guesses = read_file_lines("allowed-guesses.txt")
        .into_iter()
        .map(|line| Word::new(&line))
        .collect::<Vec<_>>();
    let allowed_answers = read_file_lines("wordle-answers.txt")
        .into_iter()
        .map(|line| Word::new(&line))
        .collect::<Vec<_>>();

    let status = WordleStatus::with_data(
        [None, None, None, None, None],
        [vec![], vec![], vec!['a'], vec!['t', 'n'], vec!['t']],
        vec!['r', 'o', 'e', 'c', 'l', 'i'],
    );

    let best_guess = status.best_guess(&allowed_guesses, &allowed_answers);

    println!("Best guess: {}", best_guess.to_string());
    println!(
        "Remaining possible answers: {:?}",
        status
            .get_possible_answers(&allowed_answers)
            .into_iter()
            .map(|word| word.to_string())
            .collect::<Vec<_>>()
    );
}
