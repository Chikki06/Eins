/// Enum giving a reason for why an action was invalid
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidAction {
    UnknownUsername,
    NotPlayerTurn,
    CannotPlayCard,
    CardNotInHand,
    GameIsOver,
}
