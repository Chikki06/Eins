use crate::{
    asset::{card_back_image, get_card_image},
    game_state::Game,
};
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, Align, Align2, Area, CentralPanel, Frame, Grid, ImageButton, Layout, RichText,
    ScrollArea, SidePanel, Spacing, Ui, Vec2, Widget,
};
use tokio::task::JoinHandle;

use crate::{
    client::{ClientConnection, ServerData},
    game_state::card::{Card, CardColor, ColorlessCard},
    protocol::{GameState, ServerState},
};

#[derive(Debug)]
struct DropThread(JoinHandle<()>);
impl Drop for DropThread {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Debug, Default)]
pub struct Menu {
    join_address: String,
    error_message: Option<String>,
    join_thread: Option<DropThread>,
}

#[derive(Debug, Default)]
pub struct SetUpLobby {
    username: String,
    port: String,
    error_message: Option<String>,
    host_thread: Option<DropThread>,
}

pub struct Connected {
    username: String,
    connection: ClientConnection,
}

pub enum Page {
    Menu(Menu),
    SetUpLobby(SetUpLobby),
    Connected(Connected),
}

impl Default for Page {
    fn default() -> Self {
        Self::Menu(Menu::default())
    }
}

#[derive(Clone, Default)]
pub struct GameUI {
    page: Arc<Mutex<Page>>,
}

impl Menu {
    fn render(&mut self, game: GameUI, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Join Address:");
            ui.text_edit_singleline(&mut self.join_address);

            if ui.button("Join").clicked() {
                if self.join_address.is_empty() {
                    self.error_message = Some("Address cannot be empty".to_string());
                } else {
                    let address = self.join_address.clone();
                    let game = game.clone();
                    self.join_thread = Some(DropThread(tokio::spawn(async move {
                        let result = ClientConnection::join_server(&address).await;
                        let mut page = game.page.lock().unwrap();
                        match result {
                            Ok(connection) => {
                                *page = Page::Connected(Connected {
                                    username: "".into(),
                                    connection,
                                })
                            }
                            Err(reason) => {
                                if let Page::Menu(menu) = &mut *page {
                                    menu.error_message = Some(reason.to_string());
                                }
                            }
                        }
                    })));
                }
            }
        });

        ui.label("OR");
        if ui.button("Host a Server").clicked() {
            let game = game.clone();
            tokio::spawn(async move {
                let mut page = game.page.lock().unwrap();
                *page = Page::SetUpLobby(SetUpLobby::default());
            });
        }

        if let Some(message) = &self.error_message {
            ui.label(message);
        }
    }
}

impl SetUpLobby {
    fn render(&mut self, game: GameUI, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
        });

        ui.horizontal(|ui| {
            ui.label("Port:");
            egui::TextEdit::singleline(&mut self.port)
                .hint_text("Blank for automatic port")
                .ui(ui);
        });

        ui.horizontal(|ui| {
            if ui.button("Create Server").clicked() {
                if self.username.is_empty() {
                    self.error_message = Some("Username cannot be empty".to_string());
                } else {
                    let parsed_port = if self.port.is_empty() {
                        Ok(0)
                    } else {
                        self.port.parse::<u16>()
                    };
                    match parsed_port {
                        Ok(port) => {
                            let username = self.username.clone();
                            let game = game.clone();
                            self.host_thread = Some(DropThread(tokio::spawn(async move {
                                let result = ClientConnection::host_server(&username, port).await;
                                let mut page = game.page.lock().unwrap();
                                match result {
                                    Ok(connection) => {
                                        *page = Page::Connected(Connected {
                                            username,
                                            connection,
                                        })
                                    }
                                    Err(reason) => {
                                        if let Page::SetUpLobby(lobby) = &mut *page {
                                            lobby.error_message = Some(reason.to_string());
                                        }
                                    }
                                }
                            })));
                        }
                        Err(_) => self.error_message = Some("Invalid port".to_string()),
                    };
                }
            }
            if ui.button("Cancel").clicked() {
                let game = game.clone();
                tokio::spawn(async move {
                    let mut page = game.page.lock().unwrap();
                    *page = Page::Menu(Menu::default());
                });
            }
        });

        if let Some(message) = &self.error_message {
            ui.label(message);
        }
    }
}

impl Connected {
    fn render(&mut self, game: GameUI, ctx: &egui::Context, ui: &mut Ui) {
        let server_data = {
            let lock = self.connection.server_state.lock().unwrap();
            lock.clone()
        };
        match &server_data.server_state {
            Some(server_state) => match &server_state.game_state {
                Some(game_state) => {
                    self.playing(game, &server_state, game_state, ctx, ui);
                }
                None => {
                    self.in_lobby(game, &server_state, ctx, ui);
                }
            },
            None => {
                self.input_username(game, &server_data, ctx, ui);
            }
        }
    }

    fn input_username(
        &mut self,
        game: GameUI,
        server_data: &ServerData,
        ctx: &egui::Context,
        ui: &mut Ui,
    ) {
        ui.heading("Connected to server.");
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            if ui.button("Join").clicked() {
                self.connection.register_username(self.username.clone());
            }
        });

        if ui.button("Leave").clicked() {
            let game = game.clone();
            tokio::spawn(async move {
                let mut page = game.page.lock().unwrap();
                *page = Page::Menu(Menu::default());
            });
        }

        if let Some(error) = &server_data.last_error {
            ui.label(format!("{:?}", error));
        }
    }

    fn in_lobby(
        &mut self,
        game: GameUI,
        server_state: &ServerState,
        ctx: &egui::Context,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            ui.heading("In lobby.");
            if ui.button("Leave").clicked() {
                let game = game.clone();
                tokio::spawn(async move {
                    let mut page = game.page.lock().unwrap();
                    *page = Page::Menu(Menu::default());
                });
            }
        });
        ui.label(format!("Your name: {}", server_state.client_username));

        if self.connection.is_hosting() {
            ui.label(format!("Hosting on {}", self.connection.host_address()));
            if ui.button("Start Game").clicked() {
                self.connection.start_game();
            }
        }

        ui.label("");
        ui.label("Players connected:");
        for username in &server_state.usernames {
            ui.label(username.clone());
        }
    }

    fn playing(
        &mut self,
        game: GameUI,
        server_state: &ServerState,
        game_state: &GameState,
        ctx: &egui::Context,
        ui: &mut Ui,
    ) {
        let is_turn = game_state.current_player == server_state.client_username;
        if let Some(winner_name) = game_state.card_counts.iter().find_map(|(name, &count)| {
            if count == 0 { Some(name.clone()) } else { None }
        }) {
            egui::Window::new("Winner!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("🎉 Winner! 🎉");
                        ui.label(winner_name);
                        if ui.button("Leave").clicked() {
                            let game = game.clone();
                            tokio::spawn(async move {
                                let mut page = game.page.lock().unwrap();
                                *page = Page::Menu(Menu::default());
                            });
                        }
                    });
                });
        }
        // Players sidebar
        SidePanel::left("players").resizable(false).show(ctx, |ui| {
            // show play‐direction above the list
            let dir = if game_state.is_clockwise {
                "Down"
            } else {
                "Up"
            };
            ui.label(format!("Direction: {}", dir));

            ui.heading("Players");
            for name in &game_state.player_order {
                let count = game_state.card_counts.get(name).unwrap_or(&0);
                ui.horizontal(|ui| {
                    // player name
                    let mut name_text = RichText::new(name.clone());
                    if *name == server_state.client_username {
                        name_text = name_text.strong();
                    }
                    ui.label(name_text);

                    // 2) indicate whose turn it is
                    if name == &game_state.current_player {
                        ui.label(
                            RichText::new(format!(
                                "◀ {} turn",
                                if is_turn { "Your" } else { "Their" }
                            ))
                            .strong(),
                        );
                    }

                    // card count, right-aligned
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(count.to_string());
                    });
                });
            }
            if ui.button("Leave").clicked() {
                let game = game.clone();
                tokio::spawn(async move {
                    let mut page = game.page.lock().unwrap();
                    *page = Page::Menu(Menu::default());
                });
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            // Top‐of‐pile + Active Color
            ui.vertical_centered(|ui| {
                ui.heading("Card Pile");

                ui.columns_const(|[col_1, col_2]| {
                    col_1.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        ui.add(get_card_image(
                            &game_state.top_card,
                            &game_state.active_color,
                        ));
                        ui.shrink_width_to_current();
                        ui.vertical_centered(|ui| {
                            ui.label("Top Card");
                        });
                    });
                    col_2.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        if ui
                            .add_enabled(is_turn, ImageButton::new(card_back_image()).frame(false))
                            .clicked()
                        {
                            let _ = self.connection.draw_card();
                        }
                        ui.shrink_width_to_current();
                        ui.vertical_centered(|ui| {
                            ui.label("Draw Card");
                        });
                    });
                });
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Your Hand");

                let mut sorted: Vec<_> = game_state.hand.iter().collect();
                sorted.sort();
                let hand: Vec<_> = sorted
                    .into_iter()
                    .flat_map(|(card, count)| std::iter::repeat_n(card, (*count) as usize))
                    .collect();

                let card_size = card_back_image()
                    .load_and_calc_size(ui, Vec2::default())
                    .unwrap_or(Vec2::splat(100.0));
                let spacing = card_size.x + Spacing::default().item_spacing.x;
                let num_columns = ((ui.max_rect().width() / spacing) as usize)
                    .max(1)
                    .min(hand.len());

                ui.vertical_centered(|ui| {
                    ui.set_max_width((num_columns as f32) * spacing);
                    ui.columns(num_columns, |columns| {
                        for col in 0..num_columns {
                            for index in (col..hand.len()).step_by(num_columns) {
                                let card = hand[index];
                                let playable = Game::can_play_card(
                                    &game_state.top_card,
                                    &game_state.active_color,
                                    &card,
                                );
                                if columns[col]
                                    .add_enabled(
                                        is_turn && playable,
                                        egui::Button::image(get_card_image(&card, &None))
                                            .frame(false),
                                    )
                                    .clicked()
                                {
                                    let _ = self.connection.play_card(card.clone());
                                }
                            }
                        }
                    });
                });

                ui.set_min_width(ui.max_rect().width());
            });
        });
    }
}

impl eframe::App for GameUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut page = self.page.lock().unwrap();
            match &mut *page {
                Page::Menu(state) => state.render(self.clone(), ctx, ui),
                Page::SetUpLobby(state) => state.render(self.clone(), ctx, ui),
                Page::Connected(state) => state.render(self.clone(), ctx, ui),
            };
        });
    }
}
