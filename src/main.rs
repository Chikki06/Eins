mod asset;
mod client;
mod debug_cli;
mod external_connection;
mod game_state;
mod protocol;
mod server;
mod ui;

use std::env;

use debug_cli::debug_cli;
use eframe::egui::Vec2;
use tokio::signal;
use ui::GameUI;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let identifier = args.get(1).cloned().unwrap_or("".into());
    match identifier.as_str() {
        "server" => {
            println!("Starting server.");
            let server = server::Server::standalone()
                .await
                .expect("Error ocurred while starting server.");
            let _ = signal::ctrl_c().await;
            drop(server);
            println!("Disconnected.");
        }
        "" | "client" => {
            // debug_cli();
            let mut options = eframe::NativeOptions::default();
            options.viewport.min_inner_size = Some(Vec2::splat(600.0));
            options.viewport.maximized = Some(true);
            eframe::run_native(
                "Eins",
                options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);

                    Ok(Box::<GameUI>::default())
                }),
            )
            .expect("Failed to run eframe native application")
        }
        _ => panic!("Argument 1 should be 'server' or 'client'"),
    }
}
