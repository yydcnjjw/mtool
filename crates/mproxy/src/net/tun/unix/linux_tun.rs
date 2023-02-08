use std::{
    ffi::CStr,
    mem,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    ptr,
};

use anyhow::Context;
use libc::{c_char, c_int, c_uint, ifreq, IFNAMSIZ};
use nix::{fcntl::OFlag, sys::stat::Mode};

nix::ioctl_read!(tungetiff, b'T', 210, c_uint);
nix::ioctl_write_int!(tunsetiff, b'T', 202);

pub struct Tun {
    pub fd: OwnedFd,
}

impl Tun {
    pub fn new(name: &str, flags: i32) -> Result<Tun, anyhow::Error> {
        let fd = nix::fcntl::open(
            "/dev/net/tun",
            OFlag::O_RDWR | OFlag::O_NONBLOCK,
            Mode::empty(),
        )?;

        let mut req: ifreq = unsafe { mem::zeroed() };

        let len = name.len().min(IFNAMSIZ as usize - 1);
        let name = &name[..len];

        unsafe {
            ptr::copy_nonoverlapping(
                name.as_ptr().cast::<c_char>(),
                req.ifr_name.as_mut_ptr(),
                len,
            );
        }

        req.ifr_ifru.ifru_flags = flags as i16;

        unsafe {
            tunsetiff(fd, &req as *const _ as _).context("Failed to ioctl TUNSETIFF")?;
        }

        Ok(Tun {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        })
    }

    pub fn ifname(&self) -> Result<String, anyhow::Error> {
        Ok(unsafe {
            CStr::from_ptr(self.get_iff()?.ifr_name.as_ptr())
                .to_string_lossy()
                .into()
        })
    }

    pub fn ifindex(&self) -> Result<c_int, anyhow::Error> {
        Ok(unsafe { self.get_iff()?.ifr_ifru.ifru_ifindex })
    }

    fn get_iff(&self) -> Result<ifreq, anyhow::Error> {
        let mut req: ifreq = unsafe { mem::zeroed() };

        unsafe {
            tungetiff(
                self.fd.as_raw_fd(),
                (&mut req as *mut ifreq).cast::<c_uint>(),
            )
            .context("Failed to ioctl TUNGETIFF")?;
        };
        Ok(req)
    }
}

impl AsRawFd for Tun {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd.as_raw_fd()
    }
}

#[cfg(test)]
mod tests {

    use libc::{IFF_MULTI_QUEUE, IFF_NO_PI, IFF_TUN};

    use super::*;

    #[test]
    fn test_tun_create() {
        Tun::new("tun%d", IFF_TUN | IFF_NO_PI | IFF_MULTI_QUEUE).unwrap();
    }

    #[test]
    fn test_tun_get_iff() {
        let tun = Tun::new("tun%d", IFF_TUN | IFF_NO_PI | IFF_MULTI_QUEUE).unwrap();
        let ifname = tun.ifname().unwrap();
        println!("{}", ifname);
    }
}
