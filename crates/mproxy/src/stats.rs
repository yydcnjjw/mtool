use std::{
    collections::HashMap,
    fmt, ops,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Weak,
    },
};

use tokio::sync::RwLock;
use weak_table::PtrWeakHashSet;

#[derive(Debug, Clone, Default)]
pub struct TransferStats {
    pub tx: usize,
    pub rx: usize,
}

impl TransferStats {
    pub fn new() -> Self {
        Self { tx: 0, rx: 0 }
    }
}

impl ops::SubAssign for TransferStats {
    fn sub_assign(&mut self, rhs: Self) {
        if self.tx > rhs.tx {
            self.tx -= rhs.tx;
        } else {
            self.tx = 0;
        }

        if self.rx > rhs.rx {
            self.rx -= rhs.rx;
        } else {
            self.rx = 0;
        }
    }
}

impl ops::AddAssign for TransferStats {
    fn add_assign(&mut self, rhs: Self) {
        self.tx += rhs.tx;
        self.rx += rhs.rx;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub transfer: HashMap<String, TransferStats>,
}

pub trait GetTransferStats: Send + Sync {
    fn get_transfer_stats(&self) -> TransferStats;
}

pub struct TransferMonitor {
    c: RwLock<PtrWeakHashSet<Weak<dyn GetTransferStats>>>,
}

impl fmt::Debug for TransferMonitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransferMonitor").finish()
    }
}

impl TransferMonitor {
    pub fn new() -> Self {
        Self {
            c: RwLock::new(PtrWeakHashSet::new()),
        }
    }

    pub async fn get_transfer_stats(&self) -> Result<TransferStats, anyhow::Error> {
        let mut s = TransferStats::new();

        for elem in self.c.read().await.iter() {
            s += elem.get_transfer_stats();
        }
        Ok(s)
    }

    pub async fn bind(&self, v: Arc<dyn GetTransferStats>) {
        self.c.write().await.insert(v);
    }
}

#[derive(Debug, Clone)]
pub struct Copyed(Arc<AtomicU64>, Arc<AtomicU64>);

impl Copyed {
    pub fn new((tx, rx): (Arc<AtomicU64>, Arc<AtomicU64>)) -> Self {
        Self(tx, rx)
    }
}

impl GetTransferStats for Copyed {
    fn get_transfer_stats(&self) -> TransferStats {
        TransferStats {
            tx: self.0.load(Ordering::Relaxed) as usize,
            rx: self.1.load(Ordering::Relaxed) as usize,
        }
    }
}
