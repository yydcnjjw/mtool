use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TransferStats {
    pub tx: usize,
    pub rx: usize,
}

impl ops::SubAssign for TransferStats {
    fn sub_assign(&mut self, rhs: Self) {
        if self.tx >= rhs.tx {
            self.tx -= rhs.tx;
        }

        if self.rx >= rhs.tx {
            self.rx -= rhs.rx;
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Stats {
    pub transfer: BTreeMap<String, TransferStats>,
}
