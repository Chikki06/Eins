mod server_error;

use futures_util::StreamExt;
use local_ip_address::local_ip;
pub use server_error::ServerError;

use bimap::BiMap;
use futures_channel::mpsc::{UnboundedReceiver, unbounded};

use std::{collections::HashSet, sync::Arc};

use tokio::{net::TcpListener, sync::Mutex};
use tokio::{net::TcpStream, task::JoinHandle};

use crate::external_connection::{self, Sender};
use crate::protocol::ManageServer;
use crate::{
    game_state::{Game, card::Card},
    protocol::{self, ClientToServer, ErrorResponse, ServerToClient},
};

#[derive(Debug, Default)]
pub struct ServerState {
    current_connections: BiMap<String, Sender<ServerToClient>>,
    usernames: HashSet<String>,
    game: Option<Game>,
}

impl ServerState {
    fn new(host_username: String, host_sender: Sender<ServerToClient>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            current_connections: BiMap::from_iter([(host_username.clone(), host_sender)]),
            usernames: HashSet::from([host_username]),
            game: None,
        }))
    }

    /// Attempts to register the username for a sender
    /// If the username has an existing connection, returns an error
    /// If the sender already has a username, returns an error
    /// If the game has already begun, returns an error
    /// Otherwise, adds the user to self.usernames and self.current_connections
    fn register_username(
        &mut self,
        sender: &Sender<ServerToClient>,
        username: &String,
    ) -> Result<(), ErrorResponse> {
        // Check if username is already in use
        if self.usernames.contains(username) && self.current_connections.contains_left(username) {
            return Err(ErrorResponse::UsernameInUse);
        }

        // Check if sender already has a username
        if self.current_connections.contains_right(sender) {
            return Err(ErrorResponse::InvalidAction);
        }

        // Check if game has already begun
        if self.game.is_some() && !self.usernames.contains(username) {
            return Err(ErrorResponse::GameHasBegun);
        }

        // Add user to usernames and current_connections
        self.usernames.insert(username.clone());
        self.current_connections
            .insert(username.clone(), sender.clone());

        Ok(())
    }

    /// Attempts to play a card for a sender
    /// If the sender is not in the game, returns an error
    /// If the card cannot be played, returns an error
    /// Otherwise, plays the card
    fn play_card(
        &mut self,
        sender: &Sender<ServerToClient>,
        card: &Card,
    ) -> Result<(), ErrorResponse> {
        let username = self
            .current_connections
            .get_by_right(sender)
            .ok_or(ErrorResponse::NotInGame)?;

        let game = self.game.as_mut().ok_or(ErrorResponse::GameNotStarted)?;

        match game.play_card(username, card) {
            Ok(()) => Ok(()),
            Err(_) => Err(ErrorResponse::InvalidAction),
        }
    }

    /// Attempts to draw a card for a sender
    /// If the sender is not in the game, returns an error
    /// If the card cannot be drawn, returns an error
    /// Otherwise, draws a card
    fn draw_card(&mut self, sender: &Sender<ServerToClient>) -> Result<(), ErrorResponse> {
        let username = self
            .current_connections
            .get_by_right(sender)
            .ok_or(ErrorResponse::NotInGame)?;

        let game = self.game.as_mut().ok_or(ErrorResponse::GameNotStarted)?;

        match game.draw_card(username) {
            Ok(()) => Ok(()),
            Err(_) => Err(ErrorResponse::InvalidAction),
        }
    }

    /// Attempts to remove a sender from the game
    /// If the sender is not in the game, returns an error
    /// Otherwise, removes the sender from self.current_connections
    /// If the game has not begun, also remove their username from self.usernames
    fn remove_sender(&mut self, sender: &Sender<ServerToClient>) -> Result<(), ErrorResponse> {
        // Get and copy username before any mutable operations
        let username = self
            .current_connections
            .get_by_right(sender)
            .ok_or(ErrorResponse::NotInGame)?
            .clone();

        // Remove from current_connections
        self.current_connections.remove_by_right(sender);

        // If game hasn't started, also remove from usernames set
        if self.game.is_none() {
            self.usernames.remove(&username);
        }

        Ok(())
    }

    /// Returns the server state for this username
    /// Returns None if the username is not connected
    fn get_user_server_state(&self, username: &String) -> Option<protocol::ServerState> {
        // If the username isn't connected, return None
        if !self.current_connections.contains_left(username) {
            return None;
        }

        // Get game state if game exists
        let game_state = self
            .game
            .as_ref()
            .map(|game| game.get_player_game_state(username))
            .flatten();

        Some(protocol::ServerState {
            game_state,
            usernames: self.usernames.clone(),
            client_username: username.clone(),
        })
    }

    /// If message is Ok, sends an updated server state to all connected users
    /// If message is Err, sends an error response to only sender
    fn send_update(
        &self,
        source: &Option<Sender<ServerToClient>>,
        message: &Result<(), ErrorResponse>,
    ) {
        match message {
            Ok(()) => {
                // On success, send updated state to all connected users
                for (username, sender) in self.current_connections.iter() {
                    if let Some(state) = self.get_user_server_state(username) {
                        let _ = sender.unbounded_send(ServerToClient::StateUpdate(state));
                    }
                }
            }
            Err(error) => {
                if let Some(sender) = source {
                    // On error, send error only to source
                    let _ = sender.unbounded_send(ServerToClient::Error(error.clone()));
                }
            }
        }
    }

    fn start_game(&mut self) -> Result<(), ErrorResponse> {
        if self.game.is_some() {
            return Err(ErrorResponse::GameAlreadyStarted);
        }
        self.game = Some(
            Game::new(&Vec::from_iter(self.usernames.clone()), 6)
                .ok_or(ErrorResponse::FailedToStartGame)?,
        );
        Ok(())
    }
}

async fn handle_request(
    server_state: Arc<Mutex<ServerState>>,
    sender: Sender<ServerToClient>,
    message: ClientToServer,
) {
    let mut state = server_state.lock().await;
    let result = match message {
        ClientToServer::RegisterUsername(username) => state.register_username(&sender, &username),
        ClientToServer::PlayCard(card) => state.play_card(&sender, &card),
        ClientToServer::DrawCard => state.draw_card(&sender),
    };
    state.send_update(&Some(sender), &result);
}

async fn handle_management(
    server_state: Arc<Mutex<ServerState>>,
    host_sender: Option<Sender<ServerToClient>>,
    message: ManageServer,
) {
    let mut state = server_state.lock().await;
    let result = match message {
        ManageServer::StartGame => state.start_game(),
    };
    state.send_update(&host_sender, &result);
}

async fn handle_connection(
    server_state: Arc<Mutex<ServerState>>,
    stream: TcpStream,
) -> Result<(), ServerError> {
    let stream = tokio_tungstenite::accept_async(stream)
        .await
        .or(Err(ServerError::FailedToAccept))?;
    let (map_sender, map_receiver) = unbounded::<ServerToClient>();
    let sender = Sender::new(map_sender);
    external_connection::handle_connection(
        stream,
        &sender,
        map_receiver,
        |message: ClientToServer, sender| handle_request(server_state.clone(), sender, message),
    )
    .await;

    {
        let mut state = server_state.lock().await;
        let result = state.remove_sender(&sender);
        state.send_update(&Some(sender), &result);
    }

    Ok(())
}

#[derive(Debug)]
pub struct Server {
    server_state: Arc<Mutex<ServerState>>,
    host_sender: Option<Sender<ServerToClient>>,
    connection_handles: Vec<JoinHandle<()>>,
    host_address: String,
}

impl Server {
    async fn make(
        address: &(String, u16),
        server_state: Arc<Mutex<ServerState>>,
        host_sender: Option<Sender<ServerToClient>>,
        mut handles: Vec<JoinHandle<()>>,
    ) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(address)
            .await
            .or(Err(ServerError::FailedToBind))?;

        let port = match listener.local_addr() {
            Ok(ip) => match ip {
                std::net::SocketAddr::V4(v4) => v4.port(),
                std::net::SocketAddr::V6(v6) => v6.port(),
            },
            Err(_) => 0,
        };
        let ip = match local_ip() {
            Ok(address) => address.to_string(),
            Err(_) => "".into(),
        };

        let loop_clone = server_state.clone();
        handles.push(tokio::spawn(async move {
            loop {
                if let Ok((connection, _)) = listener.accept().await {
                    tokio::spawn(handle_connection(loop_clone.clone(), connection));
                }
            }
        }));

        Ok(Self {
            server_state,
            host_sender,
            connection_handles: handles,
            host_address: format!("{ip}:{port}"),
        })
    }

    pub async fn new(
        host_username: &String,
        host_client_receiver: UnboundedReceiver<ClientToServer>,
        address: &(String, u16),
    ) -> Result<(Self, UnboundedReceiver<ServerToClient>), ServerError> {
        let (host_stream, host_sink) = unbounded::<ServerToClient>();
        let host_sender = Sender::new(host_stream);

        let server_state = ServerState::new(host_username.clone(), host_sender.clone());

        let state_clone = server_state.clone();
        let sender_clone = host_sender.clone();
        let handle = tokio::spawn(async move {
            host_client_receiver
                .for_each(|message| {
                    let state_clone = state_clone.clone();
                    let sender = sender_clone.clone();
                    async move {
                        handle_request(state_clone, sender, message).await;
                    }
                })
                .await;
        });

        {
            // We must tell the server to replicate the host's state
            let state = server_state.lock().await;
            state.send_update(&None, &Ok(()));
        }

        Ok((
            Self::make(address, server_state, Some(host_sender), vec![handle]).await?,
            host_sink,
        ))
    }

    pub async fn standalone() -> Result<Self, ServerError> {
        Self::make(&("0.0.0.0".into(), 8080), Arc::default(), None, vec![]).await
    }

    pub async fn start_game(&self) {
        handle_management(
            self.server_state.clone(),
            self.host_sender.clone(),
            ManageServer::StartGame,
        )
        .await;
    }

    pub fn host_address(&self) -> String {
        self.host_address.clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        for handle in &self.connection_handles {
            handle.abort();
        }
    }
}
