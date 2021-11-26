pub type Sender = broadcast::Sender<Event>;
pub type Receiver = broadcast::Receiver<Event>;

pub struct EventBus {
    tx: Sender,
}

impl EventBus {
    pub fn new(cap: usize) -> Self {
        let (tx, _) = broadcast::channel(cap);
        Self { tx }
    }

    pub fn sender(&self) -> Sender {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> Receiver {
        self.tx.subscribe()
    }
}
