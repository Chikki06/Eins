use eframe::egui::{Color32, Image, include_image};

use crate::game_state::card::{Card, CardColor, ColorableType, ColorlessCard};

fn get_color32(color: &CardColor) -> Color32 {
    match color {
        CardColor::Orange => Color32::from_rgb(240, 183, 61),
        CardColor::Blue => Color32::from_rgb(140, 225, 252),
        CardColor::Green => Color32::from_rgb(30, 245, 62),
        CardColor::Purple => Color32::from_rgb(237, 101, 246),
    }
}

pub fn get_card_image(card: &Card, active_color: &Option<CardColor>) -> Image<'static> {
    let image = match card {
        Card::Colored(color, card_type) => (
            match card_type {
                ColorableType::Zero => include_image!("../assets/CardZero.png"),
                ColorableType::One => include_image!("../assets/CardOne.png"),
                ColorableType::Two => include_image!("../assets/CardTwo.png"),
                ColorableType::Three => include_image!("../assets/CardThree.png"),
                ColorableType::Four => include_image!("../assets/CardFour.png"),
                ColorableType::Five => include_image!("../assets/CardFive.png"),
                ColorableType::Six => include_image!("../assets/CardSix.png"),
                ColorableType::Seven => include_image!("../assets/CardSeven.png"),
                ColorableType::Eight => include_image!("../assets/CardEight.png"),
                ColorableType::Nine => include_image!("../assets/CardNine.png"),
                ColorableType::PlusThree => include_image!("../assets/CardPlusThree.png"),
                ColorableType::Reverse => include_image!("../assets/CardReverse.png"),
                ColorableType::Skip => include_image!("../assets/CardSkip.png"),
            },
            get_color32(color),
        ),
        Card::Colorless(card_type) => match active_color {
            Some(color) => (
                match card_type {
                    ColorlessCard::Random => include_image!("../assets/CardRandomBlank.png"),
                    ColorlessCard::RandomPlusSix => {
                        include_image!("../assets/CardRandomPlusSixBlank.png")
                    }
                },
                get_color32(color),
            ),
            None => (
                match card_type {
                    ColorlessCard::Random => include_image!("../assets/CardRandom.png"),
                    ColorlessCard::RandomPlusSix => {
                        include_image!("../assets/CardRandomPlusSix.png")
                    }
                },
                Color32::WHITE,
            ),
        },
    };
    Image::new(image.0).tint(image.1).fit_to_original_size(1.0)
}

pub fn card_back_image() -> Image<'static> {
    Image::new(include_image!("../assets/CardBack.png")).fit_to_original_size(1.0)
}
