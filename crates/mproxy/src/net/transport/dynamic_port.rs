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
    T: Connect<S>, {
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

        let this = Self {
            address: Address::from_str(&address)?,
            start,
            end,
            current: AtomicU16::new(start),
            connector,
            _phantom: PhantomData,
        };

        let endpoint = this.endpoint()?;
        if let Err(e) = this.connector.connect(endpoint.clone()).await {
            warn!("connect with {} failed: {:?}", endpoint, e);
            this.set_next_endpoint()?;
        }

        Ok(this)
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

    fn set_next_endpoint(&self) -> Result<(), anyhow::Error> {
        if (self.current.fetch_add(1, Ordering::Relaxed) + 1) > self.end {
            anyhow::bail!("{}-{} have been exhausted", self.start, self.end);
        }
        Ok(())
    }

    pub async fn connect(&self) -> Result<S, anyhow::Error> {
        Ok(match self.connector.open_stream().await {
            Ok(s) => s,
            Err(e) => {
                warn!("open stream failed: {:?}", e);

                loop {
                    self.set_next_endpoint()?;
                    let endpoint = self.endpoint()?;
                    match self.connector.connect(endpoint.clone()).await {
                        Ok(_) => break,
                        Err(e) => {
                            warn!("connect with {} failed: {:?}", endpoint, e);
                        }
                    }
                }

                self.connector.open_stream().await?
            }
        })
    }
}
