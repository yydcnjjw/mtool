use std::{
    marker::PhantomData,
    net::SocketAddr,
    str::FromStr,
    sync::atomic::{AtomicU16, Ordering},
};

use anyhow::Context;
use tracing::warn;

use crate::{config::transport::Endpoint, proxy::Address};

use super::Connect;

#[derive(Debug)]
pub struct Connector<T, S> {
    address: Address,
    start: u16,
    end: u16,
    current: AtomicU16,
    connector: T,
    _phantom: PhantomData<S>,
}

impl<T, S> Connector<T, S>
where
    T: Connect<S>,
{
    pub async fn new(connector: T, endpoint: Endpoint) -> Result<Self, anyhow::Error> {
        let (address, start, end) = match endpoint {
            Endpoint::Single { address, port } => (address, port, port),
            Endpoint::Multi {
                address,
                port_range,
            } => {
                let (start, end) = port_range
                    .split_once("-")
                    .context(format!("{} is incorrect", port_range))?;

                let (start, end) = (
                    u16::from_str_radix(start, 10)?,
                    u16::from_str_radix(end, 10)?,
                );
                (address, start, end)
            }
        };

        Ok(Self {
            address: Address::from_str(&address)?,
            start,
            end,
            current: AtomicU16::new(start),
            connector,
            _phantom: PhantomData,
        })
    }
}

impl<T, S> Connector<T, S>
where
    T: Connect<S>,
{
    fn endpoint(&self) -> Result<SocketAddr, anyhow::Error> {
        Ok(SocketAddr::from_str(&format!(
            "{}:{}",
            self.address.to_string(),
            self.current.load(Ordering::Relaxed)
        ))?)
    }

    fn set_next_endpoint(&self) {
        if (self.current.fetch_add(1, Ordering::Relaxed) + 1) > self.end {
            self.current.store(self.start, Ordering::Relaxed);
        }
    }

    pub async fn connect(&self) -> Result<S, anyhow::Error> {
        loop {
            if !self.connector.is_open().await {
                let endpoint = self.endpoint()?;
                match self.connector.connect(endpoint.clone()).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("connect with {} failed: {:?}", endpoint, e);
                        self.connector.close().await;
                        self.set_next_endpoint();
                        continue;
                    }
                }
            }

            match self.connector.open_stream().await {
                Ok(s) => return Ok(s),
                Err(e) => {
                    warn!("open stream failed: {:?}", e);
                    self.connector.close().await;
                    self.set_next_endpoint();
                    continue;
                }
            }
        }
    }
}
