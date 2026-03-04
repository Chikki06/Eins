use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::game_state::card::{Card, CardColor};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CardList(Vec<(Card, i64)>);

impl From<HashMap<Card, i64>> for CardList {
    fn from(value: HashMap<Card, i64>) -> Self {
        Self(value.into_iter().collect())
    }
}

impl Into<HashMap<Card, i64>> for CardList {
    fn into(self) -> HashMap<Card, i64> {
        return self.0.into_iter().collect();
    }
}

mod card_list_serde {
    use std::collections::HashMap;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::game_state::card::Card;

    pub fn serialize<S>(map: &HashMap<Card, i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        map.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Card, i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Vec::<(Card, i64)>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {
    UsernameInUse,
    GameIsFull,
    GameHasBegun,
    NotInGame,
    GameNotStarted,
    InvalidAction,
    GameAlreadyStarted,
    FailedToStartGame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub current_player: String,
    pub is_clockwise: bool,
    pub top_card: Card,
    pub winner: Option<String>,
    pub active_color: Option<CardColor>,
    pub card_counts: HashMap<String, i64>,
    pub player_order: Vec<String>,
    #[serde(with = "card_list_serde")]
    pub hand: HashMap<Card, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub game_state: Option<GameState>,
    pub usernames: HashSet<String>,
    pub client_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClient {
    StateUpdate(ServerState),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientToServer {
    RegisterUsername(String),
    PlayCard(Card),
    DrawCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManageServer {
    StartGame,
}
