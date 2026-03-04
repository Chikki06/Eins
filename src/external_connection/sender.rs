use std::{
    hash::Hash,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures_channel::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct Sender<T> {
    id: usize,
    sender: UnboundedSender<T>,
}

impl<T> PartialEq for Sender<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sender: self.sender.clone(),
        }
    }
}

impl<T> Eq for Sender<T> {}

impl<T> Hash for Sender<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Sender<T> {
    pub fn new(sender: UnboundedSender<T>) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            sender,
        }
    }
}

impl<T> Deref for Sender<T> {
    type Target = UnboundedSender<T>;
    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}
