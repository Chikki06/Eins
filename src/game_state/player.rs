// Kalin

use std::{collections::HashMap, hash::Hash};

use super::card::Card;

/// Represents a player inside of a game.
#[derive(Debug)]
pub struct Player {
    hand: HashMap<Card, i64>,
    active: bool,
    username: String,
}

impl Player {
    pub fn new(username: String, starting_hand_size: i64) -> Self {
        let mut hand = HashMap::new();
        for _ in 0..starting_hand_size {
            hand.entry(Card::random())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        Self {
            hand,
            active: true,
            username,
        }
    }

    pub fn make_active(&mut self) {
        self.active = true;
    }

    pub fn make_inactive(&mut self) {
        self.active = false;
    }

    pub fn add_cards(&mut self, count: i64) {
        for _ in 0..count {
            self.hand.entry(Card::random())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    pub fn has_card(&self, card: &Card) -> bool {
        self.hand.contains_key(card)
    }

    pub fn remove_card(&mut self, card: &Card) {
        if let Some(count) = self.hand.get_mut(card) {
            *count -= 1;
            if *count == 0 {
                self.hand.remove(card);
            }
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn hand(&self) -> &HashMap<Card, i64> {
        &self.hand
    }
}

// Testing-only functions to allow for the construction of non-random games
#[cfg(test)]
impl Player {
    pub fn with_hand(username: String, hand: HashMap<Card, i64>) -> Self {
        Self {
            hand,
            active: true,
            username,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game_state::card::{CardColor, ColorableType, ColorlessCard};

    use super::*;

    const BLUE_CARD: Card = Card::Colored(CardColor::Blue, ColorableType::One);
    const GREEN_CARD: Card = Card::Colored(CardColor::Green, ColorableType::One);
    const RANDOM_CARD: Card = Card::Colorless(ColorlessCard::Random);
    const RANDOM_PLUS_6_CARD: Card = Card::Colorless(ColorlessCard::RandomPlusSix);

    fn make_test_player() -> Player {
        Player {
            hand: HashMap::from([(BLUE_CARD, 5), (GREEN_CARD, 3), (RANDOM_CARD, 1)]),
            active: true,
            username: "foo".into(),
        }
    }

    fn count_cards(player: &Player) -> i64 {
        player.hand.iter().fold(0, |sum, item| sum + item.1)
    }

    #[test]
    fn test_add_cards() {
        let mut player = make_test_player();

        let initial = count_cards(&player);
        player.add_cards(5);
        assert!(count_cards(&player) == initial + 5);
        player.add_cards(100);
        assert!(count_cards(&player) == initial + 100);
    }

    #[test]
    fn test_has_card() {
        let player = make_test_player();

        assert!(player.has_card(&BLUE_CARD));
        assert!(player.has_card(&GREEN_CARD));
        assert!(player.has_card(&RANDOM_CARD));
        assert!(!player.has_card(&RANDOM_PLUS_6_CARD));
    }

    #[test]
    fn test_remove_card() {
        let mut player = make_test_player();

        player.remove_card(&BLUE_CARD);
        assert!(player.hand == HashMap::from([(BLUE_CARD, 4), (GREEN_CARD, 3), (RANDOM_CARD, 1)]));
        player.remove_card(&RANDOM_CARD);
        assert!(player.hand == HashMap::from([(BLUE_CARD, 4), (GREEN_CARD, 3)]));
    }
}
