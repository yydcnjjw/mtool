use std::{
    io::{self, Read, Write},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use anyhow::Context;

pub struct TunIO {
    fd: OwnedFd,
}

impl TunIO {
    pub fn new<T: AsRawFd>(fd: T) -> Result<Self, anyhow::Error> {
        Ok(Self {
            fd: unsafe {
                OwnedFd::from_raw_fd(nix::unistd::dup(fd.as_raw_fd()).context("TunIO: dup tun fd")?)
            },
        })
    }
}

impl AsRawFd for TunIO {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd.as_raw_fd()
    }
}

impl Read for TunIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = unsafe {
            libc::read(
                self.fd.as_raw_fd(),
                buf.as_mut_ptr() as *mut _,
                buf.len() as _,
            )
        };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(n as _)
    }
}

impl Write for TunIO {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = unsafe {
            libc::write(
                self.fd.as_raw_fd(),
                buf.as_ptr() as *const _,
                buf.len() as _,
            )
        };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(n as _)
    }

    fn flush(&mut self) -> io::Result<()> {
        let ret = unsafe { libc::fsync(self.fd.as_raw_fd()) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
