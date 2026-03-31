use std::io::{stdin,stdout,Write};
use std::fmt::{Display, Formatter, Error};

use chacha20::ChaCha20Rng;
use rand::RngExt;
use rand::rand_core::{SeedableRng};

pub fn get_input_fn(label: String) -> String {
    let mut s = String::new();
    print!("{}", label);
    let _ = stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    s
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    One,
    Two,
    Three,
    Four,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Color::One => write!(f, "1"),
            Color::Two => write!(f, "2"),
            Color::Three => write!(f, "3"),
            Color::Four => write!(f, "4"),
        }
    }
}

impl Color {
    fn rand(rng: &mut ChaCha20Rng) -> Color {
        let y: u8 = rng.random_range(1..=4);
        match y {
            1 => Color::One,
            2 => Color::Two,
            3 => Color::Three,
            4 => Color::Four,
            other => panic!("Color::rand: expected 1-4, but found {other}")
        }
    }

    pub fn count(&self) -> [u8; 4] {
        match self {
            Color::One => [1, 0, 0, 0],
            Color::Two => [0, 1, 0, 0],
            Color::Three => [0, 0, 1, 0],
            Color::Four => [0, 0, 0, 1],
        }
    }

    pub fn combine_counts(xs: [u8; 4], ys: [u8; 4]) -> [u8; 4] {
        xs.into_iter().zip(ys.into_iter()).map(|(x, y)| {
            x + y
        }).collect::<Vec<_>>().try_into().expect("expected that zipping two arrays of 4 elements would be 4 elements")
    }

    pub fn compare_counts(goal_counts: [u8; 4], guess_counts: [u8; 4]) -> [u8; 4] {
        goal_counts.into_iter().zip(guess_counts.into_iter()).map(|(goal_count, guess_count)| {
            // num_correct_guess = largest x: x <= guess_count and x <= goal_count ??
            // if guess_count == 0, then 0
            // if goal_count == 1, guess_count == 1, then 1
            // if goal_count == 2, guess_count == 1, then 1
            // if goal_count == 1, guess_count == 2, then 1
            std::cmp::min(goal_count, guess_count)
        }).collect::<Vec<_>>().try_into().expect("expected that zipping two arrays of 4 elements would be 4 elements")
    }
}

// - AI chooses 4 colors 1-4
// - player guess_rows up to 10 times 4 colors 1-4
// - after each guess:
//     + 1-4 "correct, wrong spot"
//     + 1-4 "correct, right spot"
#[derive(Debug, PartialEq, Eq)]
pub struct GameState {
    rng: ChaCha20Rng,
    goal_row: [Color; 4],
    guess_rows: Vec<[Color; 4]>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for guess_row in self.guess_rows.iter() {
            let score = self.score_one(*guess_row);
            writeln!(
                f,
                "{} | {} | {} | {} : {}/4 correct, {}/4 wrong spot",
                guess_row[0],
                guess_row[1],
                guess_row[2],
                guess_row[3],
                score.right_spot,
                score.wrong_spot,
            )?;
        }
        Ok(())
    }
}

impl GameState {
    fn new(seed: [u8; 32]) -> GameState {
        let mut rng = ChaCha20Rng::from_seed(seed);
        let goal_row: [Color; 4] = [(); 4].map(|_| { Color::rand(&mut rng) });
        let guess_rows = Vec::new();
        GameState {
            rng,
            goal_row,
            guess_rows,
        }
    }

    pub fn guess(&mut self, guess_row: [Color; 4]) {
        self.guess_rows.push(guess_row);
    }

    pub fn guess_io(&mut self) -> [Color; 4] { 
        let input = get_input_fn("> ".to_string());
        let guess_row = input.chars().map(|c| {
            match c {
                '1' => Color::One,
                '2' => Color::Two,
                '3' => Color::Three,
                '4' => Color::Four,
                other => panic!("expected 1, 2, 3, or 4, but found: {}", other),
            }
        }).collect::<Vec<_>>().try_into().expect("expected 4 inputs, but found another amount");
        self.guess(guess_row);
        guess_row
    }

    pub fn score_one(&self, guess_row: [Color; 4]) -> Score {
        let mut score = Score::default();
        let (leftover_goals, leftover_guess_rows): (Vec<Option<Color>>, Vec<Option<Color>>) = self.goal_row.iter().zip(guess_row).map(|(goal_item, guess_item)| {
            if *goal_item == guess_item {
                score.right_spot += 1;
                (None, None)
            } else {
                (Some(*goal_item), Some(guess_item))
            }
        }).unzip();

        let leftover_goal_counts: [u8; 4] = leftover_goals.iter().fold([0; 4], |acc, opt_goal_item| {
            match opt_goal_item {
                None => acc,
                Some(goal_item) => Color::combine_counts(acc, goal_item.count()),
            }
        });
        let leftover_guess_counts: [u8; 4] = leftover_guess_rows.iter().fold([0; 4], |acc, opt_guess_item| {
            match opt_guess_item {
                None => acc,
                Some(guess_item) => Color::combine_counts(acc, guess_item.count()),
            }
        });

        score.wrong_spot = Color::compare_counts(leftover_goal_counts, leftover_guess_counts).into_iter().sum();
        score
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


fn main() {
    let seed = [42u8; 32];
    let mut game_state = GameState::new(seed);
    println!("{:?}", game_state.goal_row);
    for _ in 0..10 {
        let guess_row = game_state.guess_io();
        println!("{game_state}");

        if game_state.score_one(guess_row).right_spot == 4 {
            panic!("you won!");
        }

        println!("--------------------------------------------------------");
    }
}
