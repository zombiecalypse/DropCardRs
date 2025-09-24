use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use js_sys::Math;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Card {
    pub front: String,
    pub back: String,
    pub x: f64,
    pub y: f64,
    pub flipped: bool,
}

#[wasm_bindgen]
pub struct Game {
    cards: Vec<Card>,
    width: f64,
    height: f64,
    score: i32,
    time_since_last_card: f64,
    card_spawn_interval: f64,
}

fn normalize_string(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[wasm_bindgen]
impl Game {
    pub fn new(width: f64, height: f64) -> Game {
        let mut game = Game {
            cards: vec![],
            width,
            height,
            score: 0,
            time_since_last_card: 0.0,
            card_spawn_interval: 3.0, // spawn a card every 3 seconds
        };
        game.spawn_card();
        game
    }

    pub fn tick(&mut self, dt: f64) {
        self.time_since_last_card += dt;
        if self.time_since_last_card > self.card_spawn_interval {
            self.spawn_card();
            self.time_since_last_card = 0.0;
        }

        let card_speed = 50.0; // pixels per second
        for card in self.cards.iter_mut() {
            card.y += card_speed * dt;
            if !card.flipped && card.y >= self.height - 50.0 { // 50 is card height
                card.flipped = true;
            }
        }

        // Remove cards that are off screen
        self.cards.retain(|card| card.y < self.height);
    }

    fn spawn_card(&mut self) {
        let (front, back) = self.get_random_card_data();
        self.cards.push(Card {
            front,
            back,
            x: (Math::random() * (self.width - 100.0)), // 100 is card width
            y: 0.0,
            flipped: false,
        });
    }

    fn get_random_card_data(&self) -> (String, String) {
        let data = vec![
            ("Bore da", "Good morning"),
            ("Prynhawn da", "Good afternoon"),
            ("Nos da", "Good night"),
            ("Sut mae?", "How are you?"),
            ("Croeso", "Welcome"),
        ];
        let index = (Math::random() * data.len() as f64).floor() as usize;
        (data[index].0.to_string(), data[index].1.to_string())
    }

    pub fn get_cards(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.cards).unwrap()
    }



    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn submit_answer(&mut self, answer: &str) -> bool {
        let mut correct = false;
        let mut card_to_remove_index: Option<usize> = None;

        let normalized_answer = normalize_string(answer);
        for (i, card) in self.cards.iter().enumerate() {
            if !card.flipped && normalize_string(&card.back) == normalized_answer {
                correct = true;
                card_to_remove_index = Some(i);
                break;
            }
        }

        if let Some(index) = card_to_remove_index {
            self.cards.remove(index);
            self.score += 1;
        }

        correct
    }
}
