mod client_error;

use std::sync::{Arc, Mutex as StdMutex};

use client_error::ClientError;
use futures_channel::mpsc::unbounded;
use futures_util::StreamExt;
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_tungstenite::connect_async;

use crate::{
    external_connection::{handle_connection, Sender},
    game_state::card::Card,
    protocol::{ClientToServer, ErrorResponse, ServerState, ServerToClient},
    server::Server,
};

#[derive(Debug, Default, Clone)]
pub struct ServerData {
    pub server_state: Option<ServerState>,
    pub last_error: Option<ErrorResponse>,
}

#[derive(Debug)]
pub struct ClientConnection {
    pub server_state: Arc<StdMutex<ServerData>>,
    connection_handle: JoinHandle<()>,
    sender: Sender<ClientToServer>,
    server: Option<Arc<Mutex<Server>>>,
    host_address: Option<String>,
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        self.connection_handle.abort();
    }
}

async fn update_server_state(server_state: Arc<StdMutex<ServerData>>, message: ServerToClient) {
    let mut update = server_state.lock().unwrap();
    match message {
        ServerToClient::StateUpdate(state) => update.server_state = Some(state),
        ServerToClient::Error(error) => update.last_error = Some(error),
    }
}

impl ClientConnection {
    pub async fn join_server(address: &String) -> Result<Self, ClientError> {
        let server_state: Arc<StdMutex<ServerData>> = Arc::default();

        let (connection, _) = connect_async(format!("ws://{address}"))
            .await
            .or(Err(ClientError::FailedToConnect))?;
        let (map_sender, map_receiver) = unbounded::<ClientToServer>();
        let sender = Sender::new(map_sender);
        let cloned_sender = sender.clone();
        let cloned_state = server_state.clone();
        let handle = tokio::spawn(async move {
            handle_connection(
                connection,
                &cloned_sender,
                map_receiver,
                |message: ServerToClient, _| update_server_state(cloned_state.clone(), message),
            )
            .await;
        });

        Ok(Self {
            server_state,
            connection_handle: handle,
            sender,
            server: None,
            host_address: None,
        })
    }

    pub async fn host_server(username: &String, port: u16) -> Result<Self, ClientError> {
        let (map_sender, map_receiver) = unbounded::<ClientToServer>();

        let server = Server::new(&username, map_receiver, &("0.0.0.0".into(), port))
            .await
            .or(Err(ClientError::FailedToCreateServer))?;
        let server_state: Arc<StdMutex<ServerData>> = Arc::default();

        let cloned_state = server_state.clone();
        let handle = tokio::spawn(async move {
            server
                .1
                .for_each(|message| update_server_state(cloned_state.clone(), message))
                .await;
        });

        let host_address = server.0.host_address();

        Ok(Self {
            server_state,
            connection_handle: handle,
            sender: Sender::new(map_sender),
            server: Some(Arc::new(Mutex::new(server.0))),
            host_address: Some(host_address),
        })
    }

    pub fn is_hosting(&self) -> bool {
        self.server.is_some()
    }

    pub fn host_address(&self) -> String {
        self.host_address.as_ref().unwrap_or(&"".into()).clone()
    }

    pub fn register_username(&self, username: String) {
        let _ = self
            .sender
            .unbounded_send(ClientToServer::RegisterUsername(username));
    }

    pub fn draw_card(&self) {
        let _ = self.sender.unbounded_send(ClientToServer::DrawCard);
    }

    pub fn play_card(&self, card: Card) {
        let _ = self.sender.unbounded_send(ClientToServer::PlayCard(card));
    }

    pub fn start_game(&self) {
        if let Some(server) = &self.server {
            let server = server.clone();
            tokio::spawn(async move {
                let server = server.lock().await;
                server.start_game().await;
            });
        }
    }
}
