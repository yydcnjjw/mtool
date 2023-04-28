use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::ready;
use tokio::io::{AsyncRead, AsyncWrite};

use super::CopyBuffer;

pub enum TransferState {
    Running(CopyBuffer),
    ShuttingDown(u64),
    Done(u64),
}

pub struct CopyBidirectional<'a, A: ?Sized, B: ?Sized> {
    a: &'a mut A,
    b: &'a mut B,
    a_to_b: TransferState,
    b_to_a: TransferState,
}

impl<'a, A: ?Sized, B: ?Sized> CopyBidirectional<'a, A, B> {
    pub fn new(a: &'a mut A, b: &'a mut B) -> Self {
        Self {
            a,
            b,
            a_to_b: TransferState::Running(CopyBuffer::new()),
            b_to_a: TransferState::Running(CopyBuffer::new()),
        }
    }

    pub fn copyed_ref(&self) -> (Arc<AtomicU64>, Arc<AtomicU64>) {
        (
            Self::copyed_ref_from_state(&self.a_to_b),
            Self::copyed_ref_from_state(&self.b_to_a),
        )
    }

    fn copyed_ref_from_state(s: &TransferState) -> Arc<AtomicU64> {
        match s {
            TransferState::Running(buf) => buf.copyed_ref(),
            TransferState::ShuttingDown(n) => Arc::new(AtomicU64::new(*n)),
            TransferState::Done(n) => Arc::new(AtomicU64::new(*n)),
        }
    }

    fn copyed(&self) -> (u64, u64) {
        (
            Self::copyed_from_state(&self.a_to_b),
            Self::copyed_from_state(&self.b_to_a),
        )
    }

    fn copyed_from_state(s: &TransferState) -> u64 {
        match s {
            TransferState::Running(buf) => buf.copyed(),
            TransferState::ShuttingDown(n) => *n,
            TransferState::Done(n) => *n,
        }
    }
}

fn transfer_one_direction<A, B>(
    cx: &mut Context<'_>,
    state: &mut TransferState,
    r: &mut A,
    w: &mut B,
) -> Poll<io::Result<u64>>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut r = Pin::new(r);
    let mut w = Pin::new(w);

    loop {
        match state {
            TransferState::Running(buf) => {
                let count = ready!(buf.poll_copy(cx, r.as_mut(), w.as_mut()))?;
                *state = TransferState::ShuttingDown(count);
            }
            TransferState::ShuttingDown(count) => {
                ready!(w.as_mut().poll_shutdown(cx))?;

                *state = TransferState::Done(*count);
            }
            TransferState::Done(count) => return Poll::Ready(Ok(*count)),
        }
    }
}

impl<'a, A, B> Future for CopyBidirectional<'a, A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<(u64, u64), (io::Error, (u64, u64))>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Unpack self into mut refs to each field to avoid borrow check issues.
        let copyed = self.copyed();
        let CopyBidirectional {
            a,
            b,
            a_to_b,
            b_to_a,
        } = &mut *self;

        let a_to_b =
            transfer_one_direction(cx, a_to_b, &mut *a, &mut *b).map_err(|e| (e, copyed))?;
        let b_to_a =
            transfer_one_direction(cx, b_to_a, &mut *b, &mut *a).map_err(|e| (e, copyed))?;

        // It is not a problem if ready! returns early because transfer_one_direction for the
        // other direction will keep returning TransferState::Done(count) in future calls to poll
        let a_to_b = ready!(a_to_b);
        let b_to_a = ready!(b_to_a);

        Poll::Ready(Ok((a_to_b, b_to_a)))
    }
}

// pub async fn copy_bidirectional<A, B>(a: &mut A, b: &mut B) -> Result<(u64, u64), std::io::Error>
// where
//     A: AsyncRead + AsyncWrite + Unpin + ?Sized,
//     B: AsyncRead + AsyncWrite + Unpin + ?Sized,
// {
//     CopyBidirectional {
//         a,
//         b,
//         a_to_b: TransferState::Running(CopyBuffer::new()),
//         b_to_a: TransferState::Running(CopyBuffer::new()),
//     }
//     .await
// }
