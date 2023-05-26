use std::{fmt, marker::Unpin, sync::Arc};

use anyhow::Context;
use futures::FutureExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info_span, Instrument};

use crate::{
    io::{BoxedAsyncIO, CopyBidirectional},
    stats::{Copyed, TransferMonitor},
};

pub struct UdpForwarder {
    pub stream: BoxedAsyncIO,
}

impl fmt::Debug for UdpForwarder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForwardUdpConn").finish()
    }
}

impl UdpForwarder {
    pub async fn forward<StreamIO>(self, s: StreamIO) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin,
    {
        self.forward_with_monitor_inner(s, None).await
    }

    pub async fn forward_with_monitor<StreamIO>(
        self,
        s: StreamIO,
        monitor: &TransferMonitor,
    ) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin,
    {
        self.forward_with_monitor_inner(s, Some(monitor)).await
    }

    async fn forward_with_monitor_inner<StreamIO>(
        mut self,
        mut s: StreamIO,
        monitor: Option<&TransferMonitor>,
    ) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin,
    {
        let proxy = &mut self.stream;
        let remote = &mut s;
        let copy_bi = CopyBidirectional::new(proxy, remote);

        let copyed = Arc::new(Copyed::new(copy_bi.copyed_ref()));
        if let Some(monitor) = monitor {
            monitor.bind(copyed.clone()).await;
        }

        tokio::pin!(copy_bi);

        copy_bi
            .instrument(info_span!("bidirectional_transmission"))
            .map(|v| {
                Ok(match v {
                    Ok(v) => v,
                    Err((e, copyed)) => {
                        let (a, b) = copyed;
                        if a == 0 && b == 0 {
                            return Err(e);
                        } else {
                            (a, b)
                        }
                    }
                })
            })
            .await
            .context("bidirectional transmission")
    }
}
