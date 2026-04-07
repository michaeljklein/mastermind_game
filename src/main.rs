use console_error_panic_hook::set_once as set_panic_hook;
// use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::window;

use std::fmt::{Display, Error, Formatter};
use std::io::{Write, stdin, stdout};

use rand_chacha::ChaCha20Rng;
use rand::RngExt;
use rand::rand_core::SeedableRng;

type E = String;

pub fn get_input_fn(label: String) -> Result<String, E> {
    let mut s = String::new();
    print!("{}", label);
    let _ = stdout().flush();
    match stdin().read_line(&mut s) {
        Ok(_) => (),
        Err(err) => return Err(format!("error: {err}")),
    }
    // stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Color::One => write!(f, "1"),
            Color::Two => write!(f, "2"),
            Color::Three => write!(f, "3"),
            Color::Four => write!(f, "4"),
            Color::Five => write!(f, "5"),
            Color::Six => write!(f, "6"),
        }
    }
}

impl Color {
    fn rand(rng: &mut ChaCha20Rng) -> Result<Color, E> {
        let y: u8 = rng.random_range(1..=6);
        match y {
            1 => Ok(Color::One),
            2 => Ok(Color::Two),
            3 => Ok(Color::Three),
            4 => Ok(Color::Four),
            5 => Ok(Color::Five),
            6 => Ok(Color::Six),
            other => Err(format!("Color::rand: expected 1-6, but found {other}")),
        }
    }

    pub fn count(&self) -> [u8; 6] {
        match self {
            Color::One => [1, 0, 0, 0, 0, 0],
            Color::Two => [0, 1, 0, 0, 0, 0],
            Color::Three => [0, 0, 1, 0, 0, 0],
            Color::Four => [0, 0, 0, 1, 0, 0],
            Color::Five => [0, 0, 0, 0, 1, 0],
            Color::Six => [0, 0, 0, 0, 0, 1],
        }
    }

    pub fn combine_counts(xs: [u8; 6], ys: [u8; 6]) -> Result<[u8; 6], E> {
        let counts: Vec<_> = xs
            .into_iter()
            .zip(ys.into_iter())
            .map(|(x, y)| x + y)
            .collect();
        let counts = counts.try_into().map_err(|err| { format!("Color::combine_counts: expected that zipping two arrays of 6 elements would be 6 elements: {err:?}")})?;
        Ok(counts)
    }

    pub fn compare_counts(goal_counts: [u8; 6], guess_counts: [u8; 6]) -> Result<[u8; 6], E> {
        let counts: Vec<_> = goal_counts
            .into_iter()
            .zip(guess_counts.into_iter())
            .map(|(goal_count, guess_count)| {
                // num_correct_guess = largest x: x <= guess_count and x <= goal_count ??
                // if guess_count == 0, then 0
                // if goal_count == 1, guess_count == 1, then 1
                // if goal_count == 2, guess_count == 1, then 1
                // if goal_count == 1, guess_count == 2, then 1
                std::cmp::min(goal_count, guess_count)
            })
            .collect();
        let counts = counts.try_into().map_err(|err| { format!("Color::compare_counts: expected that zipping two arrays of 6 elements would be 6 elements: {err:?}")})?;
        Ok(counts)
    }

    pub fn stringify_vec(v: Vec<Color>) -> String {
        v.into_iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join("")
    }
}

// - AI chooses 4 colors 1-4
// - player guess_rows up to 10 times 4 colors 1-4
// - after each guess:
//     + 1-4 "correct, wrong spot"
//     + 1-4 "correct, right spot"
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    rng: ChaCha20Rng,
    goal_row: [Color; 4],
    guess_rows: Vec<[Color; 4]>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for guess_row in self.guess_rows.iter() {
            let score = self.score_one(*guess_row);
            let _ = match score {
                Ok(score) => writeln!(
                    f,
                    "{} | {} | {} | {} : {}/4 correct, {}/4 wrong spot",
                    guess_row[0],
                    guess_row[1],
                    guess_row[2],
                    guess_row[3],
                    score.right_spot,
                    score.wrong_spot,
                ),
                Err(err) => writeln!(f, "{err}"),
            };
        }
        Ok(())
    }
}

impl GameState {
    fn new(seed: [u8; 32]) -> Result<GameState, E> {
        let mut rng = ChaCha20Rng::from_seed(seed);
        let goal_row: Result<Vec<_>, E> = [(); 4].iter().map(|_| Color::rand(&mut rng)).collect();
        let goal_row: [Color; 4] = goal_row?
            .try_into()
            .map_err(|err| format!("GameState::new: {}", Color::stringify_vec(err)))?;
        let guess_rows = Vec::new();
        Ok(GameState {
            rng,
            goal_row,
            guess_rows,
        })
    }

    pub fn guess(&mut self, guess_row: [Color; 4]) {
        self.guess_rows.push(guess_row);
    }

    pub fn guess_io(&mut self) -> Result<[Color; 4], E> {
        // let input = get_input_fn("> ".to_string());
        let input = loop {
            match get_input_fn("> ".to_string()) {
                Ok(input) => break input,
                Err(err) => println!("GameState::guess_io: error when calling get_input_fn: {err}"),
            }
        };
        let guess_row: Result<Vec<_>, E> = input
            .chars()
            .map(|c| match c {
                '1' => Ok(Color::One),
                '2' => Ok(Color::Two),
                '3' => Ok(Color::Three),
                '4' => Ok(Color::Four),
                '5' => Ok(Color::Five),
                '6' => Ok(Color::Six),
                other => Err(format!("expected 1-6, but found: {}", other)),
            })
            .collect();
        let guess_row: [Color; 4] = guess_row?.try_into().map_err(|err: Vec<Color>| {
            format!(
                "GameState::guess_io: expected 4 inputs, but found another amount: {} of length {}",
                Color::stringify_vec(err.clone()),
                err.len()
            )
        })?;
        self.guess(guess_row);
        Ok(guess_row)
    }

    pub fn score_one(&self, guess_row: [Color; 4]) -> Result<Score, E> {
        let mut score = Score::default();
        let (leftover_goals, leftover_guess_rows): (Vec<Option<Color>>, Vec<Option<Color>>) = self
            .goal_row
            .iter()
            .zip(guess_row)
            .map(|(goal_item, guess_item)| {
                if *goal_item == guess_item {
                    score.right_spot += 1;
                    (None, None)
                } else {
                    (Some(*goal_item), Some(guess_item))
                }
            })
            .unzip();

        let leftover_goal_counts =
            leftover_goals
                .iter()
                .fold(Ok([0; 6]), |acc, opt_goal_item| match opt_goal_item {
                    None => acc,
                    Some(goal_item) => Color::combine_counts(acc?, goal_item.count()),
                })?;
        let leftover_guess_counts = leftover_guess_rows.iter().fold(
            Ok([0; 6]),
            |acc, opt_guess_item| match opt_guess_item {
                None => acc,
                Some(guess_item) => Color::combine_counts(acc?, guess_item.count()),
            },
        )?;

        score.wrong_spot = Color::compare_counts(leftover_goal_counts, leftover_guess_counts)?
            .into_iter()
            .sum();
        Ok(score)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Score {
    right_spot: u8,
    wrong_spot: u8,
}

impl Default for Score {
    fn default() -> Score {
        Score {
            right_spot: 0,
            wrong_spot: 0,
        }
    }
}

fn run_game() -> Result<(), E> {
    let mut seed = [42u8; 32];

    rand::fill(&mut seed[..]);

    let mut game_state = GameState::new(seed)?;
    println!("{:?}", game_state.goal_row);
    for g in 0..10 {
        println!("guess number: {}", g);
        let guess_row = loop {
            match game_state.guess_io() {
                Ok(out) => break out,
                Err(err) => println!("error: {err}"),
            }
        };
        println!("{game_state}");

        if game_state.score_one(guess_row)?.right_spot == 4 {
            println!("you won!");
            return Ok(());
        }

        println!("--------------------------------------------------------");
    }
    Ok(())
}

// TODO: WIP

#[wasm_bindgen]
pub fn make_game_state(seed_str: &str) -> String {
    // TODO: seed_str -> seed
    let mut seed = [42u8; 32];
    rand::fill(&mut seed[..]);
    let game_state = GameState::new(seed).expect("expected new GameState to succeed");
    let out_game_state_str = serde_json::to_string(&game_state).expect("expected serde_json serialization to succeed");
    out_game_state_str
}

fn run_game_step_wasm(input_str: &str, game_state: GameState) -> Result<GameState, E> {
    unimplemented!("run_game_step")
}

#[wasm_bindgen]
pub fn run_game_step(input_str: &str, game_state_str: &str) -> String {
    // TODO: pseudocode
    let game_state = serde_json::from_str(game_state_str).unwrap();
    let output_game_state = run_game_step_wasm(input_str, game_state).unwrap();
    let out_game_state_str = serde_json::to_string(&output_game_state).expect("expected serde_json serialization to succeed");
    out_game_state_str
}

// TODO: WIP
fn main() {
    set_panic_hook();
}
