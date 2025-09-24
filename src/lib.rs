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
    pub time_since_flipped: Option<f64>,
}

#[wasm_bindgen]
pub struct Game {
    cards: Vec<Card>,
    width: f64,
    height: f64,
    score: i32,
    time_since_last_card: f64,
    card_spawn_interval: f64,
    health: i32,
    max_health: i32,
    score_since_last_heart: i32,
    game_over: bool,
    paused: bool,
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
            health: 3,
            max_health: 5,
            score_since_last_heart: 0,
            game_over: false,
            paused: false,
        };
        game.spawn_card();
        game
    }

    pub fn tick(&mut self, dt: f64) {
        if self.game_over || self.paused {
            return;
        }

        self.time_since_last_card += dt;
        if self.time_since_last_card > self.card_spawn_interval {
            self.spawn_card();
            self.time_since_last_card = 0.0;
        }

        let card_speed = 50.0; // pixels per second
        for card in self.cards.iter_mut() {
            if card.flipped {
                if let Some(time) = &mut card.time_since_flipped {
                    *time += dt;
                }
            } else {
                card.y += card_speed * dt;
                if card.y >= self.height - 50.0 { // 50 is card height
                    card.y = self.height - 50.0; // Stop at the bottom
                    card.flipped = true;
                    card.time_since_flipped = Some(0.0);
                    if !self.game_over {
                        self.health -= 1;
                        if self.health <= 0 {
                            self.health = 0;
                            self.game_over = true;
                        }
                    }
                }
            }
        }

        // Remove cards that have been flipped for over 1 second
        self.cards.retain(|card| {
            if let Some(time_flipped) = card.time_since_flipped {
                time_flipped < 1.0
            } else {
                true // Keep cards that haven't been flipped
            }
        });
    }

    fn spawn_card(&mut self) {
        let (front, back) = self.get_random_card_data();
        self.cards.push(Card {
            front,
            back,
            x: (Math::random() * (self.width - 150.0)), // 150 is card width
            y: 0.0,
            flipped: false,
            time_since_flipped: None,
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

    pub fn get_health(&self) -> i32 {
        self.health
    }

    pub fn get_max_health(&self) -> i32 {
        self.max_health
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn restart(&mut self) {
        self.cards.clear();
        self.score = 0;
        self.health = 3;
        self.score_since_last_heart = 0;
        self.game_over = false;
        self.paused = false;
        self.time_since_last_card = 0.0;
        self.card_spawn_interval = 3.0;
        self.spawn_card();
    }

    pub fn submit_answer(&mut self, answer: &str) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        log(&format!("submit_answer called with: '{}'", answer));
        let normalized_answer = normalize_string(answer);
        log(&format!("Normalized answer: '{}'", normalized_answer));
        let initial_card_count = self.cards.len();

        log(&format!("Cards before removal ({}):", initial_card_count));
        for card in &self.cards {
            log(&format!("  - '{}'", card.front));
        }

        self.cards.retain(|card| {
            !(!card.flipped && normalize_string(&card.back) == normalized_answer)
        });

        let removed_count = initial_card_count - self.cards.len();

        log(&format!("Cards after removal ({}):", self.cards.len()));
        for card in &self.cards {
            log(&format!("  - '{}'", card.front));
        }
        
        if removed_count > 0 {
            log(&format!("Correct answer. Removed {} cards.", removed_count));
            let new_points = removed_count as i32;
            self.score += new_points;
            self.score_since_last_heart += new_points;

            self.card_spawn_interval = (3.0 - (self.score / 5) as f64 * 0.25).max(0.5);

            let hearts_to_gain = self.score_since_last_heart / 5;
            if hearts_to_gain > 0 {
                self.health = (self.health + hearts_to_gain).min(self.max_health);
                self.score_since_last_heart %= 5;
            }
            true
        } else {
            log("Incorrect answer.");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  HeLlO, WoRlD!  "), "hello world");
        assert_eq!(normalize_string("How are you?"), "how are you");
        assert_eq!(normalize_string("test-ing 123"), "testing 123");
    }

    #[wasm_bindgen_test]
    fn test_submit_correct_answer() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(game.submit_answer("Answer"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
        assert_eq!(game.get_score(), 1);
    }

    #[wasm_bindgen_test]
    fn test_submit_incorrect_answer() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(!game.submit_answer("Wrong"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(game.get_score(), 0);
    }

    #[wasm_bindgen_test]
    fn test_submit_answer_normalization() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q".to_string(), back: "How are you?".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert!(game.submit_answer("how are you"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }
    
    #[wasm_bindgen_test]
    fn test_submit_answer_resolves_multiple_cards() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q1".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q2".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q3".to_string(), back: "Different".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(game.submit_answer("answer"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].back, "Different");
        assert_eq!(game.get_score(), 2);
    }
    
    #[wasm_bindgen_test]
    fn test_tick_moves_stops_flips_and_vanishes() {
        let height = 800.0;
        let mut game = Game::new(600.0, height);
        game.cards = vec![
            Card { front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;

        // Tick to just before the flip threshold
        let card_speed = 50.0;
        let flip_y = height - 50.0;
        let time_to_flip = flip_y / card_speed;
        
        game.tick(time_to_flip - 0.1);
        let cards_before_flip: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_before_flip.len(), 1);
        assert!(!cards_before_flip[0].flipped);
        assert!(cards_before_flip[0].y < flip_y);

        // Tick past the flip threshold
        game.tick(0.2);
        let cards_after_flip: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(game.health, 2);
        assert_eq!(cards_after_flip.len(), 1);
        assert!(cards_after_flip[0].flipped);
        assert_eq!(cards_after_flip[0].y, flip_y);

        // Tick for another second, card should be gone
        game.tick(1.0);
        let cards_after_vanish: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_after_vanish.len(), 0);
    }
    
    #[wasm_bindgen_test]
    fn test_health_gain_on_score() {
        let mut game = Game::new(600.0, 800.0);
        game.health = 1; // set health low to test gain
        game.cards = vec![
            Card { front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert!(game.submit_answer("A"));
        assert_eq!(game.get_score(), 5);
        assert_eq!(game.get_health(), 2);
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_game_over_and_restart() {
        let mut game = Game::new(600.0, 800.0);
        game.health = 1;
        game.cards = vec![
            Card { front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;
        let height = 800.0;
        let card_speed = 50.0;
        let flip_y = height - 50.0;
        let time_to_flip = flip_y / card_speed;

        game.tick(time_to_flip + 0.1); // trigger flip and health loss
        assert!(game.is_game_over());
        assert_eq!(game.get_health(), 0);

        // in game over, tick should do nothing
        game.tick(10.0);
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1); // card is not removed

        // restart
        game.restart();
        assert!(!game.is_game_over());
        assert_eq!(game.get_health(), 3);
        assert_eq!(game.get_score(), 0);
        let cards_after_restart: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_after_restart.len(), 1); // one new card spawned
    }

    #[wasm_bindgen_test]
    fn test_pause_and_resume() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 10.0, flipped: false, time_since_flipped: None },
        ];

        game.pause();
        assert!(game.is_paused());

        // Tick should not move cards when paused
        game.tick(1.0);
        let cards_paused: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_paused[0].y, 10.0);

        // Submitting answers should do nothing when paused
        assert!(!game.submit_answer("A"));
        let cards_paused_submit: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_paused_submit.len(), 1);

        game.resume();
        assert!(!game.is_paused());

        // Tick should work again
        game.tick(1.0);
        let cards_resumed: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert!(cards_resumed[0].y > 10.0);

        // Submitting answers should work again
        assert!(game.submit_answer("A"));
        let cards_resumed_submit: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards_resumed_submit.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_card_spawn_interval_decreases_with_score() {
        let mut game = Game::new(600.0, 800.0);
        game.cards = vec![
            Card { front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.card_spawn_interval, 3.0);
        game.submit_answer("A");
        assert_eq!(game.get_score(), 5);
        assert_eq!(game.card_spawn_interval, 2.75);
    }
}
