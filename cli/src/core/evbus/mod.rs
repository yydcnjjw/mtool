pub trait Event {
    fn type_id() -> u32;
}

type DynamicEvent = Box<dyn Event>;

pub type Sender = broadcast::Sender<DynamicEvent>;
pub type Receiver = broadcast::Receiver<DynamicEvent>;

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
