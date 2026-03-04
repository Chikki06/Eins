mod sender;

pub use sender::Sender;

use std::future::Future;

use futures_channel::mpsc::UnboundedReceiver;
use futures_util::{Sink, StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::select;
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_connection<Send, Receive, E, R>(
    connection: impl StreamExt<Item = Result<Message, E>> + Sink<Message>,
    sender: &Sender<Send>,
    receiver: UnboundedReceiver<Send>,
    on_message: impl (Fn(Receive, Sender<Send>) -> R) + Copy,
) where
    Send: Serialize + DeserializeOwned,
    Receive: Serialize + DeserializeOwned,
    R: Future,
{
    let (outgoing, incoming) = connection.split();

    let to_elsewhere = receiver
        .filter_map(|client_to_server| async move {
            let as_string = serde_json::to_string(&client_to_server)
                .or_else(|reason| {
                    eprintln!("Failed to serialize message: {:?}", reason);
                    Err(())
                })
                .ok()?;
            Some(Ok(Message::text(as_string)))
        })
        .forward(outgoing);
    let from_elsewhere = incoming.try_for_each(|message| {
        let sender = sender.clone();
        async move {
            let string = match message.to_text() {
                Ok(string) => string,
                Err(_) => return Ok(()),
            };
            let decoded: Receive = match serde_json::from_str(string) {
                Ok(decoded) => decoded,
                Err(_) => return Ok(()),
            };
            on_message(decoded, sender).await;
            Ok(())
        }
    });

    select! {
        _ = to_elsewhere => {}
        _ = from_elsewhere => {}
    }
}
