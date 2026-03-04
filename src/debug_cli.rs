use std::io::{self, Write};

use crate::game_state::{
    card::{Card, CardColor, ColorableType, ColorlessCard},
    Game,
};

fn debug_get_players() -> Vec<String> {
    let mut players = vec![];
    loop {
        print!("Insert player name (leave empty to start the game): ");
        io::stdout().flush().unwrap();
        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();
        name = name.trim().into();

        if name.is_empty() {
            if players.len() < 2 {
                println!("Not enough players to start the game.");
                continue;
            }
            return players;
        }
        if players.contains(&name) {
            println!("{name} is already in the game.");
            continue;
        }
        players.push(name);
    }
}

fn debug_get_card(card_string: &str) -> Option<Card> {
    match card_string {
        "random" => return Some(Card::Colorless(ColorlessCard::Random)),
        "random_plus_six" => return Some(Card::Colorless(ColorlessCard::RandomPlusSix)),
        _ => (),
    }

    let (color, card_type_string) = if card_string.starts_with("orange_") {
        (
            CardColor::Orange,
            card_string.strip_prefix("orange_").unwrap_or(""),
        )
    } else if card_string.starts_with("purple_") {
        (
            CardColor::Purple,
            card_string.strip_prefix("purple_").unwrap_or(""),
        )
    } else if card_string.starts_with("blue_") {
        (
            CardColor::Blue,
            card_string.strip_prefix("blue_").unwrap_or(""),
        )
    } else if card_string.starts_with("green_") {
        (
            CardColor::Green,
            card_string.strip_prefix("green_").unwrap_or(""),
        )
    } else {
        return None;
    };

    let card_type = match card_type_string {
        "zero" => ColorableType::Zero,
        "one" => ColorableType::One,
        "two" => ColorableType::Two,
        "three" => ColorableType::Three,
        "four" => ColorableType::Four,
        "five" => ColorableType::Five,
        "six" => ColorableType::Six,
        "seven" => ColorableType::Seven,
        "eight" => ColorableType::Eight,
        "nine" => ColorableType::Nine,
        "plus_three" => ColorableType::PlusThree,
        "reverse" => ColorableType::Reverse,
        "skip" => ColorableType::Skip,
        _ => return None,
    };
    Some(Card::Colored(color, card_type))
}

fn debug_make_action(game: &mut Game) {
    println!("Input one of the following actions:");
    println!("\tplay <username> <card>");
    println!("\tdraw <username>");
    println!("For colored cards, put <color>_<name> (e.g. blue_one, orange_plus_three).");
    println!("For non-colored cards, put <name> (e.g. random, random_plus_six).");
    loop {
        print!("Command: ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();

        let words: Vec<_> = line.trim().split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            continue;
        }

        let result = match words[0] {
            "play" => {
                if words.len() != 3 {
                    continue;
                }
                match debug_get_card(words[2]) {
                    Some(card) => game.play_card(&words[1].into(), &card),
                    None => continue,
                }
            }
            "draw" => {
                if words.len() != 2 {
                    continue;
                }
                game.draw_card(&words[1].into())
            }
            _ => continue,
        };

        match result {
            Ok(_) => return,
            Err(reason) => {
                println!("Invalid Action: {:#?}", reason);
                continue;
            }
        }
    }
}

pub fn debug_cli() {
    let players = debug_get_players();
    let mut game = Game::new(&players, 5).unwrap();
    loop {
        println!("{:#?}", game);
        debug_make_action(&mut game);
    }
}
