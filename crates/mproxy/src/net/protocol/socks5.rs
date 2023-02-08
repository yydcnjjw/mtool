use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr},
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tracing::trace;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Cmd {
    Connect = 1,
    Bind = 2,
    UdpAssociate = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum AddrType {
    IPv4 = 1,
    Domain = 3,
    IPv6 = 4,
}

#[repr(u8)]
#[derive(Debug)]
pub enum ResponseCode {
    Success = 0,
    Failure = 1,
    RuleFailure = 2,
    NetworkUnreachable = 3,
    HostUnreachable = 4,
    ConnectionRefused = 5,
    TTLExpired = 6,
    CommandNotSupported = 7,
    AddrTypeNotSupported = 8,
}

pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn accept<S>(self, s: S) -> Result<(), anyhow::Error>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        Connection::new(s).accept().await
    }
}

pub struct Connection<S> {
    s: S,
}

impl<S> Connection<S> {
    fn new(s: S) -> Self {
        Self { s }
    }
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    async fn auth(&mut self) -> Result<(), anyhow::Error> {
        let mut buf = [0u8; 257];

        {
            let _ = self.s.read(&mut buf).await?;

            let (ver, nmethods) = (buf[0], buf[1]);
            let methods = &buf[2..(nmethods + 2) as usize];

            trace!(ver, nmethods, "{:?}", methods);
        }

        {
            buf[1] = 0; // set no authentication
            let _ = self.s.write_all(&buf[0..2]).await?;
        }
        Ok(())
    }

    // async fn send_reply(&mut self, code: ResponseCode, bind_ddr: ) {
    // }

    // async fn handle_request(&mut self) -> Result<(), anyhow::Error> {
    //     let mut buf = [0u8; 22];

    //     let _ = self.s.read(&mut buf).await?;

    //     let (_ver, cmd, atyp) = (buf[0], Cmd::try_from(buf[1])?, AddrType::try_from(buf[3])?);

    //     if let Cmd::Connect = cmd {
    //         let sockaddr = to_sockaddr(atyp, &buf[4..]);

    //         trace!(sockaddr, "accept CMD {:?}", cmd);

    //         let mut stream = TcpStream::connect(sockaddr).await?;

    //         {
    //             buf[1] = 0;
    //             buf[3] = 1;
    //             buf[4] = 0;
    //             buf[5] = 0;
    //             buf[6] = 0;
    //             buf[7] = 0;
    //             buf[8] = 0;
    //             buf[9] = 0;

    //             let _ = self.s.write_all(&buf[0..10]).await?;
    //         }

    //         {
    //             let (from_client, from_server) =
    //                 tokio::io::copy_bidirectional(&mut self.s, &mut stream).await?;

    //             trace!(
    //                 "client wrote {} bytes and received {} bytes",
    //                 from_client,
    //                 from_server
    //             );
    //         }
    //     } else {
    //         anyhow::bail!("{:?} is not supported", cmd)
    //     }

    //     Ok(())
    // }

    async fn accept(&mut self) -> Result<(), anyhow::Error> {
        self.auth().await?;
        // self.command().await?;
        Ok(())
    }
}

fn to_sockaddr(atyp: AddrType, b: &[u8]) -> String {
    match atyp {
        AddrType::IPv4 => {
            format!(
                "{}:{}",
                Ipv4Addr::new(b[0], b[1], b[2], b[3]),
                u16::from_be_bytes([b[4], b[5]])
            )
        }
        AddrType::Domain => {
            let len = b[0] as usize;
            let port = &b[len + 1..];
            format!(
                "{}:{}",
                String::from_utf8_lossy(&b[1..len + 1]),
                u16::from_be_bytes([port[0], port[1]])
            )
        }
        AddrType::IPv6 => {
            let b = b
                .chunks_exact(2)
                .map(|a| u16::from_be_bytes([a[0], a[1]]))
                .collect::<Vec<_>>();

            format!(
                "{}:{}",
                Ipv6Addr::new(b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]),
                b[8]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::net::TcpListener;
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn test_socks5_server() -> Result<(), anyhow::Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8100));

        let listener = TcpListener::bind(addr).await?;
        trace!("Listening on socks5://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;

            tokio::task::spawn(async move {
                if let Err(err) = Builder::new().accept(stream).await {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
