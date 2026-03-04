use rand::Rng;

use crate::protocol::GameState;

use super::{
    card::Card, card::CardColor, card::ColorableType, card::ColorlessCard,
    invalid_action::InvalidAction, player::Player,
};

/// The entry point for external code to handle games.
/// Provides high-level control and allows for player input,
/// without needing to know the rules of the game.
#[derive(Debug)]
pub struct Game {
    players: Vec<Player>,
    current_player: String,
    is_clockwise: bool,
    top_card: Card,
    winner: Option<String>,
    active_color: Option<CardColor>,
}

impl Game {
    pub fn new(players: &Vec<String>, starting_hand_size: i64) -> Option<Self> {
        if players.len() < 2 {
            //return None;
        }
        Some(Self {
            players: players
                .iter()
                .map(|username| Player::new(username.clone(), starting_hand_size))
                .collect(),
            current_player: players[0].clone(),
            is_clockwise: true,
            top_card: Card::random(),
            winner: None,
            active_color: None,
        })
    }

    pub fn can_play_card(
        top_card: &Card,
        active_color: &Option<CardColor>,
        query_card: &Card,
    ) -> bool {
        match (top_card, query_card) {
            // For two colored cards, allow play if either the color or type matches.
            (Card::Colored(top_color, top_type), Card::Colored(card_color, card_type)) => {
                top_color == card_color || top_type == card_type
            }
            // A colorless card (wild) can be played on any colored card.
            (Card::Colored(_, _), Card::Colorless(_)) => true,
            // When the top card is wild, use the active color for validation.
            (Card::Colorless(_), Card::Colored(card_color, _)) => {
                if let Some(active) = active_color.as_ref() {
                    card_color == active
                } else {
                    true
                }
            }

            (Card::Colorless(_), Card::Colorless(_)) => true,
        }
    }

    fn find_player_index(&self, username: &str) -> Option<usize> {
        self.players.iter().position(|p| p.username() == username)
    }

    fn advance_turn(&mut self) {
        let total = self.players.len();
        let current_index = self
            .players
            .iter()
            .position(|p| p.username() == self.current_player)
            .unwrap();
        let next_index = if self.is_clockwise {
            (current_index + 1) % total
        } else {
            (current_index + total - 1) % total
        };

        self.current_player = self.players[next_index].username().to_string();
    }

    fn next_player_index(&self, current_index: usize) -> usize {
        let total = self.players.len();
        if self.is_clockwise {
            (current_index + 1) % total
        } else {
            (current_index + total - 1) % total
        }
    }

    fn can_play_on_top(&self, card: &Card) -> bool {
        Self::can_play_card(&self.top_card, &self.active_color, card)
    }

    fn apply_special_card_effect(&mut self, player_index: usize, card: &Card) -> () {
        match card {
            // PlusThree: force the next player to draw three cards and skip their turn.
            Card::Colored(_, ColorableType::PlusThree) => {
                let next_idx = self.next_player_index(player_index);
                self.players[next_idx].add_cards(3);
                self.advance_turn();
            }
            // Skip: skip the next player's turn.
            Card::Colored(_, ColorableType::Skip) => {
                self.advance_turn();
            }
            // Reverse: reverse the play order.
            Card::Colored(_, ColorableType::Reverse) => {
                self.is_clockwise = !self.is_clockwise;
            }
            // For wild cards (colorless Random), no extra turn advancement is done here.
            Card::Colorless(ColorlessCard::Random) => {}
            // RandomPlusSix: force the next player to draw six cards, then skip their turn.
            Card::Colorless(ColorlessCard::RandomPlusSix) => {
                let next_idx = self.next_player_index(player_index);
                self.players[next_idx].add_cards(6);
                self.advance_turn();
            }
            _ => {}
        }
    }

    pub fn play_card(&mut self, player: &String, card: &Card) -> Result<(), InvalidAction> {
        if self.winner.is_some() {
            return Err(InvalidAction::GameIsOver);
        }

        let player_index = self
            .find_player_index(player)
            .ok_or(InvalidAction::UnknownUsername)?;

        if &self.current_player != player {
            return Err(InvalidAction::NotPlayerTurn);
        }

        // The player must have the card in hand.
        if !self.players[player_index].has_card(card) {
            return Err(InvalidAction::CardNotInHand);
        }

        // Validate if the card can be played on top of the current top card.
        if !self.can_play_on_top(card) {
            return Err(InvalidAction::CannotPlayCard);
        }

        // If the card is colorless, automatically choose a random color. Otherwise, update active_color based on the card's color.
        match card {
            Card::Colorless(_) => {
                let mut rng = rand::thread_rng();
                let chosen_color = match rng.gen_range(0..4) {
                    0 => CardColor::Orange,
                    1 => CardColor::Purple,
                    2 => CardColor::Blue,
                    _ => CardColor::Green,
                };
                self.active_color = Some(chosen_color);
            }
            Card::Colored(color, _) => {
                self.active_color = Some(color.clone());
            }
        }

        // Remove the card from the player's hand and update the top card.
        self.players[player_index].remove_card(card);
        self.top_card = card.clone();

        // Apply any special effects and check if the effect already advanced the turn.
        self.apply_special_card_effect(player_index, card);

        // Check for a winner.
        if self.players[player_index].hand().is_empty() {
            self.winner = Some(player.clone());
        }

        self.advance_turn();

        Ok(())
    }

    pub fn draw_card(&mut self, player: &String) -> Result<(), InvalidAction> {
        if self.winner.is_some() {
            return Err(InvalidAction::GameIsOver);
        }
        let player_index = self
            .find_player_index(player)
            .ok_or(InvalidAction::UnknownUsername)?;

        // Ensure it is the player's turn.
        if &self.current_player != player {
            return Err(InvalidAction::NotPlayerTurn);
        }

        self.players[player_index].add_cards(1);
        self.advance_turn();

        Ok(())
    }

    /// Returns the game state for this player
    /// Returns None if the player is not in the game
    pub fn get_player_game_state(&self, player: &String) -> Option<GameState> {
        let player_idx = self.find_player_index(player)?;

        Some(GameState {
            current_player: self.current_player.clone(),
            is_clockwise: self.is_clockwise,
            top_card: self.top_card.clone(),
            winner: self.winner.clone(),
            active_color: self.active_color.clone(),
            card_counts: self
                .players
                .iter()
                .map(|p| (p.username().to_string(), p.hand().values().sum()))
                .collect(),
            player_order: self
                .players
                .iter()
                .map(|p| p.username().to_string())
                .collect(),
            hand: self.players[player_idx].hand().clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const TEST_CARD: Card = Card::Colored(CardColor::Blue, ColorableType::One);
    const PLAY_CARD: Card = Card::Colored(CardColor::Green, ColorableType::One);
    const RANDOM_CARD: Card = Card::Colorless(ColorlessCard::Random);

    fn make_test_game() -> Game {
        Game {
            players: vec![
                Player::with_hand(
                    "bob".into(),
                    HashMap::from([(PLAY_CARD, 1), (RANDOM_CARD, 1)]),
                ),
                Player::with_hand("alice".into(), HashMap::from([(PLAY_CARD, 1)])),
                Player::with_hand("joe".into(), HashMap::from([(PLAY_CARD, 1)])),
            ],
            current_player: "bob".into(),
            is_clockwise: true,
            top_card: TEST_CARD,
            winner: None,
            active_color: None,
        }
    }

    #[test]
    fn test_play_card_simple() {
        let mut game = make_test_game();

        assert!(game.play_card(&"alice".into(), &PLAY_CARD) == Err(InvalidAction::NotPlayerTurn));
        assert!(game.play_card(&"foo".into(), &PLAY_CARD) == Err(InvalidAction::UnknownUsername));
        assert!(game.play_card(&"bob".into(), &TEST_CARD) == Err(InvalidAction::CardNotInHand));
        assert!(game.play_card(&"bob".into(), &PLAY_CARD) == Ok(()));
        assert!(!game.players[0].has_card(&PLAY_CARD));
        assert!(game.current_player == "alice");
        assert!(game.top_card == PLAY_CARD);
    }

    #[test]
    fn test_draw_card_simple() {
        let mut game = make_test_game();

        assert!(game.draw_card(&"alice".into()) == Err(InvalidAction::NotPlayerTurn));
        assert!(game.draw_card(&"foo".into()) == Err(InvalidAction::UnknownUsername));
        assert!(game.draw_card(&"bob".into()) == Ok(()));
        assert!(game.current_player == "alice");
        assert!(game.top_card == TEST_CARD);
    }
}
