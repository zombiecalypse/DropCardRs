use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use unidecode::unidecode;

mod cards;

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    #[default]
    Normal,
    Reverse,
    Both,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Card {
    pub id: u32,
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
    card_deck: Vec<(String, String)>,
    unlocked_cards_count: usize,
    width: f64,
    height: f64,
    score: i32,
    time_since_last_card: f64,
    card_spawn_interval: f64,
    card_speed: f64,
    health: i32,
    max_health: i32,
    score_since_last_heart: i32,
    game_over: bool,
    paused: bool,
    rng_seed: u64,
    rng: ChaCha8Rng,
    game_id: u32,
    mode: GameMode,
    next_card_id: u32,
}

fn normalize_string(s: &str) -> String {
    unidecode(s)
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Serialize)]
struct UnlockedCard<'a> {
    front: &'a str,
    back: &'a str,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            cards: vec![],
            card_deck: vec![],
            unlocked_cards_count: 0,
            width: 600.0,
            height: 800.0,
            score: 0,
            time_since_last_card: 0.0,
            card_spawn_interval: 3.0,
            card_speed: 50.0,
            health: 3,
            max_health: 5,
            score_since_last_heart: 0,
            game_over: false,
            paused: false,
            rng_seed: 0,
            rng: ChaCha8Rng::seed_from_u64(0),
            game_id: 0,
            mode: GameMode::default(),
            next_card_id: 0,
        }
    }
}

#[wasm_bindgen]
impl Game {
    pub fn new(width: f64, height: f64, seed: u64, mode: GameMode) -> Game {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let game_id = rng.random::<u32>();

        let mut game = Game {
            width,
            height,
            rng,
            rng_seed: seed,
            game_id,
            mode,
            ..Self::default()
        };
        game.spawn_card();
        game
    }

    pub fn tick(&mut self, dt: f64) {
        if self.game_over || self.paused {
            return;
        }
        self.spawn_new_cards(dt);
        self.update_cards(dt);
    }

    fn spawn_new_cards(&mut self, dt: f64) {
        self.time_since_last_card += dt;
        let max_cards = 1 + (self.score / 10) as usize;

        if self.time_since_last_card > self.card_spawn_interval && self.cards.len() < max_cards {
            self.spawn_card();
            self.time_since_last_card = 0.0;
        }
    }

    fn update_cards(&mut self, dt: f64) {
        for card in self.cards.iter_mut() {
            if card.flipped {
                if let Some(time) = &mut card.time_since_flipped {
                    *time += dt;
                }
            } else {
                card.y += self.card_speed * dt;
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
        if self.card_deck.is_empty() {
            self.replenish_deck();
        }

        if let Some((raw_front, raw_back)) = self.card_deck.pop() {
            let should_reverse = match self.mode {
                GameMode::Reverse => true,
                GameMode::Both => self.rng.random::<bool>(),
                GameMode::Normal => false,
            };
    
            let (front, back) = if should_reverse {
                (raw_back, raw_front)
            } else {
                (raw_front, raw_back)
            };
    
            self.cards.push(Card {
                id: self.next_card_id,
                front,
                back,
                x: self.rng.random_range(0.0..(self.width - 150.0)),
                y: 0.0,
                flipped: false,
                time_since_flipped: None,
            });
            self.next_card_id += 1;
        }
    }

    fn replenish_deck(&mut self) {
        let num_available_cards = (10 + (self.score / 10) * 5) as usize;
        let all_cards = cards::CARD_DATA;
        let available_cards = &all_cards[..num_available_cards.min(all_cards.len())];
        self.unlocked_cards_count = available_cards.len();

        let mut new_deck: Vec<_> = (0..3)
            .flat_map(|_| available_cards)
            .map(|&(front, back)| (front.to_string(), back.to_string()))
            .collect();

        new_deck.shuffle(&mut self.rng);

        self.card_deck = new_deck;
    }

    pub fn get_cards(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.cards).unwrap()
    }

    pub fn get_id(&self) -> u32 {
        self.game_id
    }

    pub fn get_unlocked_cards(&self) -> JsValue {
        let num_available_cards = (10 + (self.score / 10) * 5) as usize;
        let all_cards = cards::CARD_DATA;
        let available_cards_data = &all_cards[..num_available_cards.min(all_cards.len())];
        let unlocked_cards: Vec<UnlockedCard> = match self.mode {
            GameMode::Both => available_cards_data
                .iter()
                .flat_map(|(front, back)| {
                    [
                        UnlockedCard { front, back },
                        UnlockedCard { front: back, back: front },
                    ]
                })
                .collect(),
            GameMode::Normal => available_cards_data
                .iter()
                .map(|(front, back)| UnlockedCard { front, back })
                .collect(),
            GameMode::Reverse => available_cards_data
                .iter()
                .map(|(front, back)| UnlockedCard {
                    front: back,
                    back: front,
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&unlocked_cards).unwrap()
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
        self.card_deck.clear();
        self.unlocked_cards_count = 0;
        self.score = 0;
        self.health = 3;
        self.score_since_last_heart = 0;
        self.game_over = false;
        self.paused = false;
        self.time_since_last_card = 0.0;
        self.card_spawn_interval = 3.0;
        self.card_speed = 50.0;
        self.rng = ChaCha8Rng::seed_from_u64(self.rng_seed);
        self.next_card_id = 0;
        self.spawn_card();
    }

    pub fn submit_answer(&mut self, answer: &str) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        let normalized_answer = normalize_string(answer);
        let initial_card_count = self.cards.len();

        self.cards.retain(|card| !(!card.flipped && card.back.split('/').any(|ans| normalize_string(ans.trim()) == normalized_answer)));

        let removed_count = initial_card_count - self.cards.len();
        
        let correct = removed_count > 0;
        if correct {
            self.handle_correct_answer(removed_count as i32);
        }
        correct
    }

    fn handle_correct_answer(&mut self, removed_count: i32) {
        self.score += removed_count;
        self.score_since_last_heart += removed_count;

        // Check if new cards were unlocked and replenish deck if so
        let num_unlocked_cards = ((10 + (self.score / 10) * 5) as usize).min(cards::CARD_DATA.len());
        if num_unlocked_cards > self.unlocked_cards_count {
            self.replenish_deck();
        }

        // Update difficulty
        self.card_spawn_interval = (3.0 - (self.score / 5) as f64 * 0.25).max(0.5);
        self.card_speed = 50.0 + (self.score as f64 * 2.0);

        // Update health
        let hearts_to_gain = self.score_since_last_heart / 5;
        if hearts_to_gain > 0 {
            self.health = (self.health + hearts_to_gain).min(self.max_health);
            self.score_since_last_heart %= 5;
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
        assert_eq!(normalize_string("crème brûlée"), "creme brulee");
    }

    #[wasm_bindgen_test]
    fn test_submit_correct_answer() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "Answer1 / Answer2".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(game.submit_answer("Answer2"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
        assert_eq!(game.get_score(), 1);
    }

    #[wasm_bindgen_test]
    fn test_submit_incorrect_answer() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(!game.submit_answer("Wrong"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(game.get_score(), 0);
    }

    #[wasm_bindgen_test]
    fn test_submit_answer_normalization() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "Answer One / How are you?".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert!(game.submit_answer("  how ARE you?? "));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }
    
    #[wasm_bindgen_test]
    fn test_submit_answer_with_diacritics() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "crème brûlée".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert!(game.submit_answer("creme brulee"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_submit_answer_resolves_multiple_cards() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q1".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 1, front: "Q2".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 2, front: "Q3".to_string(), back: "Different".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(game.submit_answer("answer"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, 2);
        assert_eq!(game.get_score(), 2);
    }
    
    #[wasm_bindgen_test]
    fn test_tick_moves_stops_flips_and_vanishes() {
        let height = 800.0;
        let mut game = Game::new(600.0, height, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;

        // Tick to just before the flip threshold
        let card_speed = game.card_speed;
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
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.health = 1; // set health low to test gain
        game.cards = vec![
            Card { id: 0, front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 1, front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 2, front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 3, front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 4, front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert!(game.submit_answer("A"));
        assert_eq!(game.get_score(), 5);
        assert_eq!(game.get_health(), 2);
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_game_over_and_restart() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.health = 1;
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;
        let height = 800.0;
        let card_speed = game.card_speed;
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
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 10.0, flipped: false, time_since_flipped: None },
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
    fn test_difficulty_increases_with_score() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        game.cards = vec![
            Card { id: 0, front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 1, front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 2, front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 3, front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
            Card { id: 4, front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        assert_eq!(game.card_spawn_interval, 3.0);
        assert_eq!(game.card_speed, 50.0);

        game.submit_answer("A");

        assert_eq!(game.get_score(), 5);
        assert_eq!(game.card_spawn_interval, 2.75);
        assert_eq!(game.card_speed, 50.0 + (5.0 * 2.0));
    }

    #[wasm_bindgen_test]
    fn test_deck_replenishes_on_unlock() {
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal);
        
        // Initial state: 10 cards unlocked, deck has 30 cards, one is spawned
        assert_eq!(game.unlocked_cards_count, 10);
        assert_eq!(game.card_deck.len(), 10 * 3 - 1);

        // Score enough points to unlock more cards (score 10)
        game.score = 9; // set score to 9 to be just before the threshold
        game.cards = vec![
            Card { id: game.next_card_id, front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None },
        ];
        game.submit_answer("A");
        assert_eq!(game.get_score(), 10);

        // After scoring, new cards are unlocked, and deck is replenished.
        // 15 cards should be unlocked (10 initial + 5 new).
        // Deck should have 15 * 3 = 45 cards.
        assert_eq!(game.unlocked_cards_count, 15);
        assert_eq!(game.card_deck.len(), 15 * 3);
    }

    #[wasm_bindgen_test]
    fn test_reverse_mode_card_spawn() {
        // use a seed that is not 0 to avoid predictable first card with index 0
        let game = Game::new(600.0, 800.0, 1, GameMode::Reverse); 
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        let card = &cards[0];

        let normal_game = Game::new(600.0, 800.0, 1, GameMode::Normal);
        let normal_cards: Vec<Card> = serde_wasm_bindgen::from_value(normal_game.get_cards()).unwrap();
        let normal_card = &normal_cards[0];

        assert_eq!(card.front, normal_card.back);
        assert_eq!(card.back, normal_card.front);
    }

    #[wasm_bindgen_test]
    fn test_both_mode_card_spawn() {
        let mut game = Game::new(600.0, 800.0, 1, GameMode::Both);
        for _ in 0..20 {
            game.spawn_card();
        }
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        
        let welsh_fronts = cards.iter().filter(|c| cards::CARD_DATA.iter().any(|(f, _b)| f == &c.front)).count();
        let english_fronts = cards.len() - welsh_fronts;

        assert!(welsh_fronts > 0, "Expected some cards with Welsh on front");
        assert!(english_fronts > 0, "Expected some cards with English on front");
    }
}
