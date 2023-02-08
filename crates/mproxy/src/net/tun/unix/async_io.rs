use futures::ready;
use std::os::fd::AsRawFd;
use std::task::{self, Context, Poll};
use std::{
    io::{self, Read, Write},
    pin::Pin,
};
use tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite, ReadBuf};

use super::block_io;

pub struct TunIO {
    io: AsyncFd<block_io::TunIO>,
}

impl TunIO {
    pub fn new<T>(io: T) -> Result<Self, anyhow::Error>
    where
        T: AsRawFd,
    {
        Ok(Self {
            io: AsyncFd::new(block_io::TunIO::new(io)?)?,
        })
    }
}

impl AsyncRead for TunIO {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.io.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf.initialize_unfilled())) {
                Ok(Ok(n)) => {
                    buf.set_filled(buf.filled().len() + n);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_) => continue,
            }
        }
    }
}

impl AsyncWrite for TunIO {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> task::Poll<io::Result<usize>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.io.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> task::Poll<io::Result<()>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.io.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(result) => return Poll::Ready(result),
                Err(_) => continue,
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> task::Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use libc::{IFF_NO_PI, IFF_TUN};
    use tokio::io::AsyncReadExt;

    use crate::net::tun::Tun;

    use super::*;

    #[tokio::test]
    async fn test_io() {
        let tun = Tun::new("tun%d", IFF_TUN | IFF_NO_PI).unwrap();

        let mut io = TunIO::new(tun).unwrap();

        let mut buf = vec![0u8; 1500];
        loop {
            let n = io.read(&mut buf).await.unwrap();
            println!("{}", n);
        }
    }
}
