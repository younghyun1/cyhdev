//! TCP admission for the HTTP listeners.
//!
//! [`LimitedAcceptor`] runs before TLS, so a refused connection costs no handshake.
//! The permit travels inside the returned stream and is released only when the
//! socket closes, which also keeps upgraded WebSockets counted after hyper hands
//! the connection off.

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use axum_server::accept::Accept;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};

use crate::util::connection_limit::{ConnectionLimiter, ConnectionPermit};

type AcceptFuture<S, M> = Pin<Box<dyn Future<Output = io::Result<(S, M)>> + Send>>;

/// Wraps an inner TCP acceptor with global and per-client connection caps.
#[derive(Clone)]
pub struct LimitedAcceptor<A> {
    inner: A,
    limiter: ConnectionLimiter,
}

impl<A> LimitedAcceptor<A> {
    pub fn new(inner: A, limiter: ConnectionLimiter) -> Self {
        Self { inner, limiter }
    }
}

impl<A, S> Accept<TcpStream, S> for LimitedAcceptor<A>
where
    A: Accept<TcpStream, S>,
    A::Future: Send + 'static,
    A::Stream: Send + 'static,
    A::Service: Send + 'static,
{
    type Stream = LimitedStream<A::Stream>;
    type Service = A::Service;
    type Future = AcceptFuture<Self::Stream, Self::Service>;

    fn accept(&self, stream: TcpStream, service: S) -> Self::Future {
        let admission = stream
            .peer_addr()
            .map_err(|error| io::Error::other(format!("peer address unavailable: {error}")))
            .and_then(|peer| {
                self.limiter
                    .try_acquire(peer.ip())
                    .map_err(|rejection| io::Error::other(format!("{rejection:?}")))
            });
        let permit = match admission {
            Ok(permit) => permit,
            // Dropping the stream closes the socket before any TLS or HTTP work.
            Err(error) => return Box::pin(std::future::ready(Err(error))),
        };
        let inner = self.inner.accept(stream, service);
        Box::pin(async move {
            let (stream, service) = inner.await?;
            Ok((
                LimitedStream {
                    inner: stream,
                    _permit: permit,
                },
                service,
            ))
        })
    }
}

/// A connection stream that returns its admission slot when dropped.
pub struct LimitedStream<S> {
    inner: S,
    _permit: ConnectionPermit,
}

impl<S: AsyncRead + Unpin> AsyncRead for LimitedStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for LimitedStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
