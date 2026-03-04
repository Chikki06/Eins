use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CardColor {
    Orange,
    Purple,
    Blue,
    Green,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ColorableType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    PlusThree,
    Reverse,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ColorlessCard {
    Random,
    RandomPlusSix,
}

/// Represents an individual card in a game.
/// Each card is either Colored, and has an associated CardColor and CardType,
/// or is Colorless, and has an associated ColorlessCard.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Card {
    Colorless(ColorlessCard),
    Colored(CardColor, ColorableType),
}

//Weighted random card generation, create inputs for the weights based on a vector which represents the card distribution.
impl Card {
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 90% chance for colored cards, 10% for special cards
        if rng.gen_bool(0.9) {
            // Generate a colored card
            let color = match rng.gen_range(0..4) {
                0 => CardColor::Orange,
                1 => CardColor::Purple,
                2 => CardColor::Blue,
                _ => CardColor::Green,
            };

            // Weight the card types:
            // Numbers (0-9): 70% (7% each)
            // Action cards (PlusThree, Reverse, Skip): 30% (10% each)
            let card_type = match rng.gen_range(0..100) {
                0..=6 => ColorableType::Zero,
                7..=13 => ColorableType::One,
                14..=20 => ColorableType::Two,
                21..=27 => ColorableType::Three,
                28..=34 => ColorableType::Four,
                35..=41 => ColorableType::Five,
                42..=48 => ColorableType::Six,
                49..=55 => ColorableType::Seven,
                56..=62 => ColorableType::Eight,
                63..=69 => ColorableType::Nine,
                70..=79 => ColorableType::PlusThree,
                80..=89 => ColorableType::Reverse,
                _ => ColorableType::Skip,
            };

            Card::Colored(color, card_type)
        } else {
            // Generate a special card (Random or RandomPlusSix)
            // 60% chance for Random, 40% for RandomPlusSix
            if rng.gen_bool(0.6) {
                Card::Colorless(ColorlessCard::Random)
            } else {
                Card::Colorless(ColorlessCard::RandomPlusSix)
            }
        }
    }
}
