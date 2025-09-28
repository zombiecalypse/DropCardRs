use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use unidecode::unidecode;

mod cards;

// Game constants
const CARD_WIDTH: f64 = 150.0;
const CARD_HEIGHT: f64 = 50.0;

// Deck and card unlocking constants
const INITIAL_UNLOCKED_CARDS: usize = 10;
const SCORE_PER_CARD_UNLOCK: i32 = 10;
const CARDS_PER_UNLOCK: usize = 5;
const DECK_CARD_DUPLICATES: u32 = 3;

// Difficulty scaling constants
const INITIAL_MAX_CARDS: usize = 1;
const SCORE_PER_MAX_CARD_INCREASE: i32 = 10;
const INITIAL_SPAWN_INTERVAL: f64 = 3.0;
const MIN_SPAWN_INTERVAL: f64 = 0.5;
const SCORE_PER_SPAWN_INTERVAL_DECREASE: i32 = 5;
const SPAWN_INTERVAL_DECREASE: f64 = 0.25;
const INITIAL_CARD_SPEED: f64 = 50.0;
const CARD_SPEED_INCREASE_PER_SCORE: f64 = 2.0;

// Health and scoring constants
const SCORE_PER_HEART: i32 = 5;

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
    pub raw_front: String,
    pub raw_back: String,
    pub front: String,
    pub back: String,
    pub x: f64,
    pub y: f64,
    pub flipped: bool,
    pub time_since_flipped: Option<f64>,
    pub free_misses: u32,
}

#[wasm_bindgen]
pub struct Game {
    cards: Vec<Card>,
    missed_cards: Vec<Card>,
    card_deck: Vec<(String, String)>,
    unlocked_cards_count: usize,
    card_miss_counts: HashMap<String, u32>,
    card_success_counts: HashMap<String, u32>,
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
    speed_multiplier: f64,
    card_data: Vec<(String, String)>,
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
struct CardForDisplay<'a> {
    raw_front: &'a str,
    raw_back: &'a str,
    front: &'a str,
    back: &'a str,
    success_count: u32,
    miss_count: u32,
    is_unlocked: bool,
}

#[derive(Serialize, Deserialize)]
struct CustomCard {
    front: String,
    back: String,
}

#[derive(Serialize)]
struct RenderableCard<'a> {
    id: u32,
    front: &'a str,
    back: &'a str,
    x: f64,
    y: f64,
    flipped: bool,
    free_misses: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            cards: vec![],
            missed_cards: vec![],
            card_deck: vec![],
            unlocked_cards_count: 0,
            card_miss_counts: HashMap::new(),
            card_success_counts: HashMap::new(),
            width: 600.0,
            height: 800.0,
            score: 0,
            time_since_last_card: 0.0,
            card_spawn_interval: INITIAL_SPAWN_INTERVAL,
            card_speed: INITIAL_CARD_SPEED,
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
            speed_multiplier: 1.0,
            card_data: vec![],
        }
    }
}

// This will expand text with parentheses into multiple variations.
// E.g., "card(s)" becomes ["card", "cards"].
fn expand_parens(text: &str) -> Vec<String> {
    let re = Regex::new(r"\((.*?)\)").unwrap();
    let mut results: HashSet<String> = HashSet::new();
    results.insert(text.to_string());

    loop {
        let mut next_results = HashSet::new();
        let mut changed_in_iteration = false;

        for s in results {
            if let Some(cap) = re.captures(&s) {
                changed_in_iteration = true;
                let full_match = cap.get(0).unwrap().as_str();
                let content = cap.get(1).unwrap().as_str();

                next_results.insert(s.replacen(full_match, content, 1));
                next_results.insert(s.replacen(full_match, "", 1));
            } else {
                next_results.insert(s);
            }
        }

        results = next_results;
        if !changed_in_iteration {
            break;
        }
    }

    results.into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect()
}

fn process_side(text: &str) -> String {
    let parts: Vec<String> = text.split('/')
        .map(|s| s.trim())
        .flat_map(|part| expand_parens(part))
        .collect();
    
    // Deduplicate
    let mut unique_parts: Vec<String> = Vec::new();
    for part in parts {
        if !unique_parts.contains(&part) {
            unique_parts.push(part);
        }
    }

    unique_parts.join(" / ")
}

#[wasm_bindgen]
pub fn parse_deck(text: &str) -> JsValue {
    let cards: Vec<CustomCard> = text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let front = process_side(parts[0].trim());
                let back = process_side(parts[1].trim());
                Some(CustomCard { front, back })
            } else {
                None
            }
        })
        .collect();
    serde_wasm_bindgen::to_value(&cards).unwrap()
}

#[wasm_bindgen]
pub fn get_default_deck() -> JsValue {
    parse_deck(cards::CARD_DATA)
}

impl Game {
    fn get_available_cards_data(&self) -> &[(String, String)] {
        let num_available_cards = INITIAL_UNLOCKED_CARDS
            + (self.score / SCORE_PER_CARD_UNLOCK) as usize * CARDS_PER_UNLOCK;
        &self.card_data[..num_available_cards.min(self.card_data.len())]
    }
}

#[wasm_bindgen]
impl Game {
    pub fn new(width: f64, height: f64, seed: u64, mode: GameMode, speed_multiplier: f64, custom_deck: JsValue) -> Result<Game, JsValue> {
        let custom_cards: Vec<CustomCard> = serde_wasm_bindgen::from_value(custom_deck)?;
        let card_data: Vec<(String, String)> = custom_cards
            .into_iter()
            .map(|c| (c.front, c.back))
            .collect();

        if card_data.is_empty() {
            return Err(JsValue::from_str("Custom deck cannot be empty."));
        }

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let game_id = rng.random::<u32>();

        let mut game = Game {
            width,
            height,
            rng,
            rng_seed: seed,
            game_id,
            mode,
            speed_multiplier,
            ..Self::default()
        };
        game.card_data = card_data;
        game.card_speed *= speed_multiplier;
        game.spawn_card();
        Ok(game)
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
        let max_cards = INITIAL_MAX_CARDS + (self.score / SCORE_PER_MAX_CARD_INCREASE) as usize;

        if self.time_since_last_card > self.card_spawn_interval && self.cards.len() < max_cards {
            self.spawn_card();
            self.time_since_last_card = 0.0;
        }
    }

    fn update_cards(&mut self, dt: f64) {
        let mut health_damage = 0;
        for card in self.cards.iter_mut() {
            if card.flipped {
                if let Some(time) = &mut card.time_since_flipped {
                    *time += dt;
                }
            } else {
                card.y += self.card_speed * dt;
                if card.y >= self.height - CARD_HEIGHT {
                    card.y = self.height - CARD_HEIGHT; // Stop at the bottom
                    card.flipped = true;
                    card.time_since_flipped = Some(0.0);
                    
                    if card.free_misses == 0 {
                        health_damage += 1;
                    }
                    
                    let miss_count = self.card_miss_counts.entry(card.raw_front.clone()).or_insert(0);
                    *miss_count += 1;

                    self.missed_cards.push(card.clone());
                }
            }
        }

        if health_damage > 0 && !self.game_over {
            self.health = self.health.saturating_sub(health_damage);
            if self.health == 0 {
                self.game_over = true;
            }
        }

        // Remove cards that have been flipped for over 1 second
        self.cards.retain(|card| card.time_since_flipped.map_or(true, |time| time < 1.0));
    }

    fn spawn_card(&mut self) {
        if self.card_deck.is_empty() {
            self.replenish_deck();
        }

        if let Some((raw_front, raw_back)) = self.card_deck.pop() {
            let should_reverse =
                self.mode == GameMode::Reverse || (self.mode == GameMode::Both && self.rng.random());
    
            let (front, back) = if should_reverse {
                (raw_back.clone(), raw_front.clone())
            } else {
                (raw_front.clone(), raw_back.clone())
            };
    
            let miss_count = self.card_miss_counts.get(&raw_front).cloned().unwrap_or(0);
            let success_count = self.card_success_counts.get(&raw_front).cloned().unwrap_or(0);
            let total_interactions = miss_count + success_count;
            self.cards.push(Card {
                id: self.next_card_id,
                raw_front,
                raw_back,
                front,
                back,
                x: self.rng.random_range(0.0..(self.width - CARD_WIDTH)),
                y: 0.0,
                flipped: false,
                time_since_flipped: None,
                free_misses: (2 - total_interactions).max(0),
            });
            self.next_card_id += 1;
        }
    }

    fn replenish_deck(&mut self) {
        let available_cards = self.get_available_cards_data();

        let mut new_deck = Vec::new();
        for (front, back) in available_cards {
            let success_count = self.card_success_counts.get(front).cloned().unwrap_or(0);
            let num_duplicates = (DECK_CARD_DUPLICATES as i32 - success_count as i32).max(1) as u32;
            for _ in 0..num_duplicates {
                new_deck.push((front.clone(), back.clone()));
            }
        }

        self.unlocked_cards_count = available_cards.len();
        new_deck.shuffle(&mut self.rng);

        self.card_deck = new_deck;
    }

    pub fn get_cards(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.cards).unwrap()
    }

    pub fn get_cards_for_render(&self) -> JsValue {
        let render_cards: Vec<RenderableCard> = self.cards
            .iter()
            .map(|card| RenderableCard {
                id: card.id,
                front: &card.front,
                back: &card.back,
                x: card.x,
                y: card.y,
                flipped: card.flipped,
                free_misses: card.free_misses,
            })
            .collect();
        serde_wasm_bindgen::to_value(&render_cards).unwrap()
    }

    pub fn get_id(&self) -> u32 {
        self.game_id
    }

    pub fn get_all_cards_for_display(&self) -> JsValue {
        let all_cards_data = &self.card_data;
        let num_unlocked_cards = (INITIAL_UNLOCKED_CARDS
            + (self.score / SCORE_PER_CARD_UNLOCK) as usize * CARDS_PER_UNLOCK)
            .min(all_cards_data.len());
        let cards_for_display: Vec<CardForDisplay> = match self.mode {
            GameMode::Both => all_cards_data
                .iter()
                .enumerate()
                .flat_map(|(i, (raw_front, raw_back))| {
                    let success_count = self.card_success_counts.get(raw_front).cloned().unwrap_or(0);
                    let miss_count = self.card_miss_counts.get(raw_front).cloned().unwrap_or(0);
                    let is_unlocked = i < num_unlocked_cards;
                    [
                        CardForDisplay { raw_front, raw_back, front: raw_front, back: raw_back, success_count, miss_count, is_unlocked },
                        CardForDisplay { raw_front, raw_back, front: raw_back, back: raw_front, success_count, miss_count, is_unlocked },
                    ]
                })
                .collect(),
            GameMode::Normal | GameMode::Reverse => {
                let reverse = matches!(self.mode, GameMode::Reverse);
                all_cards_data
                    .iter()
                    .enumerate()
                    .map(|(i, (raw_front, raw_back))| {
                        let (front, back) = if reverse { (raw_back.as_str(), raw_front.as_str()) } else { (raw_front.as_str(), raw_back.as_str()) };
                        let success_count = self.card_success_counts.get(raw_front).cloned().unwrap_or(0);
                        let miss_count = self.card_miss_counts.get(raw_front).cloned().unwrap_or(0);
                        let is_unlocked = i < num_unlocked_cards;
                        CardForDisplay { raw_front, raw_back, front, back, success_count, miss_count, is_unlocked }
                    })
                    .collect()
            }
        };
        serde_wasm_bindgen::to_value(&cards_for_display).unwrap()
    }

    pub fn get_missed_cards(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.missed_cards).unwrap()
    }

    pub fn get_card_miss_counts(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.card_miss_counts).unwrap()
    }

    pub fn get_card_success_counts(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.card_success_counts).unwrap()
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
        let card_data = self.card_data.clone();
        let card_success_counts = self.card_success_counts.clone();
        let card_miss_counts = self.card_miss_counts.clone();
        *self = Self {
            width: self.width,
            height: self.height,
            rng_seed: self.rng_seed,
            game_id: self.game_id,
            mode: self.mode,
            max_health: self.max_health,
            speed_multiplier: self.speed_multiplier,
            rng: ChaCha8Rng::seed_from_u64(self.rng_seed),
            ..Self::default()
        };
        self.card_data = card_data;
        self.card_success_counts = card_success_counts;
        self.card_miss_counts = card_miss_counts;
        self.card_speed *= self.speed_multiplier;
        self.spawn_card();
    }

    pub fn submit_answer(&mut self, answer: &str) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        let normalized_answer = normalize_string(answer);

        let (removed_cards, kept_cards): (Vec<Card>, Vec<Card>) = self.cards.drain(..).partition(|card| {
            !card.flipped && card.back.split('/').any(|ans| normalize_string(ans.trim()) == normalized_answer)
        });

        self.cards = kept_cards;
        
        let correct = !removed_cards.is_empty();
        if correct {
            self.handle_correct_answer(&removed_cards);
        }
        correct
    }

    fn handle_correct_answer(&mut self, removed_cards: &[Card]) {
        let removed_count = removed_cards.len() as i32;
        self.score += removed_count;
        self.score_since_last_heart += removed_count;

        for card in removed_cards {
            let count = self.card_success_counts.entry(card.raw_front.clone()).or_insert(0);
            *count += 1;
        }

        // Check if new cards were unlocked and replenish deck if so
        let num_unlocked_cards = self.get_available_cards_data().len();
        if num_unlocked_cards > self.unlocked_cards_count {
            self.replenish_deck();
        }

        // Update difficulty
        self.card_spawn_interval = (INITIAL_SPAWN_INTERVAL
            - (self.score / SCORE_PER_SPAWN_INTERVAL_DECREASE) as f64 * SPAWN_INTERVAL_DECREASE)
            .max(MIN_SPAWN_INTERVAL);
        self.card_speed = (INITIAL_CARD_SPEED + (self.score as f64 * CARD_SPEED_INCREASE_PER_SCORE)) * self.speed_multiplier;

        // Update health
        let hearts_to_gain = self.score_since_last_heart / SCORE_PER_HEART;
        if hearts_to_gain > 0 {
            self.health = (self.health + hearts_to_gain).min(self.max_health);
            self.score_since_last_heart %= SCORE_PER_HEART;
        }
    }

    pub fn generate_anki_export(&self) -> String {
        if self.missed_cards.is_empty() {
            return "".to_string();
        }

        let mut unique_cards = Vec::new();
        let mut seen_fronts = HashSet::new();

        for card in &self.missed_cards {
            if seen_fronts.insert(&card.raw_front) {
                unique_cards.push(card);
            }
        }

        let mut content = "#separator:tab\n#html:true\n".to_string();
        let card_lines: Vec<String> = unique_cards
            .iter()
            .map(|c| format!("{}\t{}", c.raw_front, c.raw_back))
            .collect();

        content.push_str(&card_lines.join("\n"));
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn new_game_for_test(width: f64, height: f64, seed: u64, mode: GameMode, speed_multiplier: f64) -> Game {
        let deck_jsvalue = get_default_deck();
        Game::new(width, height, seed, mode, speed_multiplier, deck_jsvalue).unwrap()
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  HeLlO, WoRlD!  "), "hello world");
        assert_eq!(normalize_string("How are you?"), "how are you");
        assert_eq!(normalize_string("test-ing 123"), "testing 123");
        assert_eq!(normalize_string("crème brûlée"), "creme brulee");
    }

    #[wasm_bindgen_test]
    fn test_submit_correct_answer() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "Answer1 / Answer2".to_string(), front: "Q".to_string(), back: "Answer1 / Answer2".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(game.submit_answer("Answer2"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
        assert_eq!(game.get_score(), 1);
    }

    #[wasm_bindgen_test]
    fn test_submit_incorrect_answer() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "Answer".to_string(), front: "Q".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert_eq!(game.get_score(), 0);
        assert!(!game.submit_answer("Wrong"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(game.get_score(), 0);
    }

    #[wasm_bindgen_test]
    fn test_submit_answer_normalization() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "Answer One / How are you?".to_string(), front: "Q".to_string(), back: "Answer One / How are you?".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert!(game.submit_answer("  how ARE you?? "));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }
    
    #[wasm_bindgen_test]
    fn test_submit_answer_with_diacritics() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "crème brûlée".to_string(), front: "Q".to_string(), back: "crème brûlée".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert!(game.submit_answer("creme brulee"));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_submit_answer_resolves_multiple_cards() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q1".to_string(), raw_back: "Answer".to_string(), front: "Q1".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 1, raw_front: "Q2".to_string(), raw_back: "Answer".to_string(), front: "Q2".to_string(), back: "Answer".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 2, raw_front: "Q3".to_string(), raw_back: "Different".to_string(), front: "Q3".to_string(), back: "Different".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
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
        let mut game = new_game_for_test(600.0, height, 0, GameMode::Normal, 1.0);
        game.card_miss_counts.insert("Q".to_string(), 2); // Mark card as not new
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "A".to_string(), front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;

        // Tick to just before the flip threshold
        let card_speed = game.card_speed;
        let flip_y = height - CARD_HEIGHT;
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
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.health = 1; // set health low to test gain
        game.cards = vec![
            Card { id: 0, raw_front: "Q1".to_string(), raw_back: "A".to_string(), front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 1, raw_front: "Q2".to_string(), raw_back: "A".to_string(), front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 2, raw_front: "Q3".to_string(), raw_back: "A".to_string(), front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 3, raw_front: "Q4".to_string(), raw_back: "A".to_string(), front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 4, raw_front: "Q5".to_string(), raw_back: "A".to_string(), front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert!(game.submit_answer("A"));
        assert_eq!(game.get_score(), 5);
        assert_eq!(game.get_health(), 2);
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[wasm_bindgen_test]
    fn test_game_over_and_restart() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.health = 1;
        game.card_miss_counts.insert("Q".to_string(), 2); // Mark card as not new
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "A".to_string(), front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        // Prevent new cards from spawning during the test to isolate behavior
        game.card_spawn_interval = 1_000_000.0;
        let height = 800.0;
        let card_speed = game.card_speed;
        let flip_y = height - CARD_HEIGHT;
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
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q".to_string(), raw_back: "A".to_string(), front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 10.0, flipped: false, time_since_flipped: None, free_misses: 0 },
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
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        game.cards = vec![
            Card { id: 0, raw_front: "Q1".to_string(), raw_back: "A".to_string(), front: "Q1".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 1, raw_front: "Q2".to_string(), raw_back: "A".to_string(), front: "Q2".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 2, raw_front: "Q3".to_string(), raw_back: "A".to_string(), front: "Q3".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 3, raw_front: "Q4".to_string(), raw_back: "A".to_string(), front: "Q4".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
            Card { id: 4, raw_front: "Q5".to_string(), raw_back: "A".to_string(), front: "Q5".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        assert_eq!(game.card_spawn_interval, INITIAL_SPAWN_INTERVAL);
        assert_eq!(game.card_speed, INITIAL_CARD_SPEED);

        game.submit_answer("A");

        assert_eq!(game.get_score(), 5);
        assert_eq!(game.card_spawn_interval, INITIAL_SPAWN_INTERVAL - SPAWN_INTERVAL_DECREASE);
        assert_eq!(game.card_speed, INITIAL_CARD_SPEED + (5.0 * CARD_SPEED_INCREASE_PER_SCORE));
    }

    #[wasm_bindgen_test]
    fn test_deck_replenishes_on_unlock() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        
        // Initial state: 10 cards unlocked, deck has 30 cards, one is spawned
        assert_eq!(game.unlocked_cards_count, INITIAL_UNLOCKED_CARDS);
        assert_eq!(game.card_deck.len(), INITIAL_UNLOCKED_CARDS * DECK_CARD_DUPLICATES as usize - 1);

        // Score enough points to unlock more cards (score 10)
        game.score = 9; // set score to 9 to be just before the threshold
        game.cards = vec![
            Card { id: game.next_card_id, raw_front: "Q".to_string(), raw_back: "A".to_string(), front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 0 },
        ];
        game.submit_answer("A");
        assert_eq!(game.get_score(), 10);

        // After scoring, new cards are unlocked, and deck is replenished.
        // 15 cards should be unlocked (10 initial + 5 new).
        // Deck should have 15 * 3 = 45 cards.
        assert_eq!(game.unlocked_cards_count, INITIAL_UNLOCKED_CARDS + CARDS_PER_UNLOCK);
        assert_eq!(game.card_deck.len(), (INITIAL_UNLOCKED_CARDS + CARDS_PER_UNLOCK) * DECK_CARD_DUPLICATES as usize);
    }

    #[wasm_bindgen_test]
    fn test_reverse_mode_card_spawn() {
        // use a seed that is not 0 to avoid predictable first card with index 0
        let game = new_game_for_test(600.0, 800.0, 1, GameMode::Reverse, 1.0); 
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1);
        let card = &cards[0];

        let normal_game = new_game_for_test(600.0, 800.0, 1, GameMode::Normal, 1.0);
        let normal_cards: Vec<Card> = serde_wasm_bindgen::from_value(normal_game.get_cards()).unwrap();
        let normal_card = &normal_cards[0];

        assert_eq!(card.front, normal_card.back);
        assert_eq!(card.back, normal_card.front);
    }

    #[wasm_bindgen_test]
    fn test_both_mode_card_spawn() {
        let mut game = new_game_for_test(600.0, 800.0, 1, GameMode::Both, 1.0);
        for _ in 0..20 {
            game.spawn_card();
        }
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        
        let original_fronts: HashSet<&str> = game.card_data.iter().map(|(f, _)| f.as_str()).collect();
        let welsh_fronts = cards
            .iter()
            .filter(|c| original_fronts.contains(c.front.as_str()))
            .count();
        let english_fronts = cards.len() - welsh_fronts;

        assert!(welsh_fronts > 0, "Expected some cards with Welsh on front");
        assert!(english_fronts > 0, "Expected some cards with English on front");
    }

    #[wasm_bindgen_test]
    fn test_new_with_custom_deck_success() {
        let custom_cards = vec![
            CustomCard { front: "Hello".to_string(), back: "World".to_string() },
            CustomCard { front: "Foo".to_string(), back: "Bar".to_string() },
        ];
        let custom_deck_jsvalue = serde_wasm_bindgen::to_value(&custom_cards).unwrap();
        
        let game_result = Game::new(600.0, 800.0, 0, GameMode::Normal, 1.0, custom_deck_jsvalue);
        assert!(game_result.is_ok());
        let game = game_result.unwrap();
        
        assert_eq!(game.card_data.len(), 2);
        assert_eq!(game.card_data[0], ("Hello".to_string(), "World".to_string()));
        let cards: Vec<Card> = serde_wasm_bindgen::from_value(game.get_cards()).unwrap();
        assert_eq!(cards.len(), 1); // one card should be spawned on init
    }

    #[wasm_bindgen_test]
    fn test_new_with_custom_deck_empty() {
        let custom_cards: Vec<CustomCard> = vec![];
        let custom_deck_jsvalue = serde_wasm_bindgen::to_value(&custom_cards).unwrap();
        
        let game_result = Game::new(600.0, 800.0, 0, GameMode::Normal, 1.0, custom_deck_jsvalue);
        assert!(game_result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_restart_preserves_custom_deck() {
        let custom_cards = vec![
            CustomCard { front: "Test".to_string(), back: "Deck".to_string() },
        ];
        let custom_deck_jsvalue = serde_wasm_bindgen::to_value(&custom_cards).unwrap();
        
        let mut game = Game::new(600.0, 800.0, 0, GameMode::Normal, 1.0, custom_deck_jsvalue).unwrap();
        assert_eq!(game.card_data.len(), 1);

        game.score = 100; // change some state
        game.restart();
        
        assert_eq!(game.card_data.len(), 1);
        assert_eq!(game.card_data[0], ("Test".to_string(), "Deck".to_string()));
        assert_eq!(game.get_score(), 0);
    }

    #[wasm_bindgen_test]
    fn test_card_success_and_miss_counts() {
        let mut game = new_game_for_test(600.0, 800.0, 0, GameMode::Normal, 1.0);
        
        // --- Test miss count ---
        let card_q = "Q".to_string();
        game.cards = vec![
            Card { id: 0, raw_front: card_q.clone(), raw_back: "A".to_string(), front: "Q".to_string(), back: "A".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 2 },
        ];
        game.card_spawn_interval = 1_000_000.0; // prevent more spawns

        // let card fall
        let height = 800.0;
        let card_speed = game.card_speed;
        let flip_y = height - CARD_HEIGHT;
        let time_to_flip = flip_y / card_speed;
        game.tick(time_to_flip + 0.1);

        let miss_counts: HashMap<String, u32> = serde_wasm_bindgen::from_value(game.get_card_miss_counts()).unwrap();
        assert_eq!(*miss_counts.get(&card_q).unwrap(), 1);

        // --- Test success count ---
        let card_q2 = "Q2".to_string();
        game.cards = vec![
            Card { id: 1, raw_front: card_q2.clone(), raw_back: "A2".to_string(), front: "Q2".to_string(), back: "A2".to_string(), x: 0.0, y: 0.0, flipped: false, time_since_flipped: None, free_misses: 2 },
        ];
        
        assert!(game.submit_answer("A2"));
        let success_counts: HashMap<String, u32> = serde_wasm_bindgen::from_value(game.get_card_success_counts()).unwrap();
        assert_eq!(*success_counts.get(&card_q2).unwrap(), 1);
        
        // Check that miss count for Q is preserved
        let miss_counts_after: HashMap<String, u32> = serde_wasm_bindgen::from_value(game.get_card_miss_counts()).unwrap();
        assert_eq!(*miss_counts_after.get(&card_q).unwrap(), 1);
    }
}
