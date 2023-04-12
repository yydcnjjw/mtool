use std::fmt;

use anyhow::Context;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info_span, Instrument};

use crate::io::{BoxedAsyncIO, CopyBidirectional};

pub struct ForwardTcpConn {
    pub stream: BoxedAsyncIO,
}

impl fmt::Debug for ForwardTcpConn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForwardTcpConn").finish()
    }
}

impl ForwardTcpConn {
    pub async fn forward<StreamIO>(mut self, mut s: StreamIO) -> Result<(u64, u64), anyhow::Error>
    where
        StreamIO: AsyncRead + AsyncWrite + Unpin,
    {
        let copy_bi = CopyBidirectional::new(&mut self.stream, &mut s);

        tokio::pin!(copy_bi);

        Ok(
            match copy_bi
                .as_mut()
                .instrument(info_span!("bidirectional_transmission"))
                .await
                .context("bidirectional transmission")
            {
                Ok(v) => v,
                Err(e) => {
                    let (a, b) = copy_bi.copyed();
                    if a == 0 && b == 0 {
                        return Err(e);
                    } else {
                        (a, b)
                    }
                }
            },
        )
    }
}
