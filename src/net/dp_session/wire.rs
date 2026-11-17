//! What a session needs of a connection to another machine: a byte stream it can read, write,
//! duplicate (one handle reads while others write), cut and give time limits. TCP is one
//! implementation; the Steam relay's (`net::steam`) is another; `Pipe` joins two ends in memory
//! for the tests.

#[cfg(any(test, feature = "steam"))]
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
#[cfg(test)]
use std::sync::Arc;
#[cfg(any(test, feature = "steam"))]
use std::sync::{Condvar, Mutex};
use std::time::Duration;
#[cfg(any(test, feature = "steam"))]
use std::time::Instant;

/// A connection to one machine. Reads give `Ok(0)` or an error once the other end is gone.
pub trait Wire: Read + Write + Send {
    /// Another handle on the same connection.
    fn try_clone(&self) -> std::io::Result<Box<dyn Wire>>;
    /// Ends the connection for every handle; a blocked read returns.
    fn shutdown(&self) -> std::io::Result<()>;
    /// How long a read waits for bytes (`None`: for ever); then it fails with `TimedOut`.
    fn set_read_timeout(&self, limit: Option<Duration>) -> std::io::Result<()>;
    /// How long a write may take (`None`: for ever).
    fn set_write_timeout(&self, limit: Option<Duration>) -> std::io::Result<()>;
}

impl Wire for TcpStream {
    fn try_clone(&self) -> std::io::Result<Box<dyn Wire>> {
        TcpStream::try_clone(self).map(|s| Box::new(s) as Box<dyn Wire>)
    }

    fn shutdown(&self) -> std::io::Result<()> {
        TcpStream::shutdown(self, Shutdown::Both)
    }

    fn set_read_timeout(&self, limit: Option<Duration>) -> std::io::Result<()> {
        TcpStream::set_read_timeout(self, limit)
    }

    fn set_write_timeout(&self, limit: Option<Duration>) -> std::io::Result<()> {
        TcpStream::set_write_timeout(self, limit)
    }
}

/// Bytes waiting for one direction of a connection that is not a socket.
#[derive(Default)]
#[cfg(any(test, feature = "steam"))]
pub struct Inbox {
    state: Mutex<InboxState>,
    arrived: Condvar,
}

#[cfg(any(test, feature = "steam"))]
#[derive(Default)]
struct InboxState {
    bytes: VecDeque<u8>,
    ended: bool,
}

#[cfg(any(test, feature = "steam"))]
impl Inbox {
    /// Adds bytes for the reader.
    pub fn put(&self, data: &[u8]) {
        let mut s = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.bytes.extend(data);
        drop(s);
        self.arrived.notify_all();
    }

    /// No more bytes will come: the reader gets what is left, then the end.
    pub fn end(&self) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ended = true;
        self.arrived.notify_all();
    }

    /// Takes up to `out.len()` bytes, waiting up to `limit` for the first; `Ok(0)` at the end.
    pub fn take(&self, out: &mut [u8], limit: Option<Duration>) -> std::io::Result<usize> {
        let deadline = limit.map(|l| Instant::now() + l);
        let mut s = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loop {
            if !s.bytes.is_empty() {
                let n = out.len().min(s.bytes.len());
                for (slot, byte) in out.iter_mut().zip(s.bytes.drain(..n)) {
                    *slot = byte;
                }
                return Ok(n);
            }
            if s.ended {
                return Ok(0);
            }
            s = match deadline {
                None => self
                    .arrived
                    .wait(s)
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return Err(std::io::ErrorKind::TimedOut.into());
                    }
                    self.arrived
                        .wait_timeout(s, deadline - now)
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .0
                }
            };
        }
    }
}

/// One end of an in-memory connection: reads what the other end wrote.
#[derive(Clone)]
#[cfg(test)]
pub struct Pipe {
    read: Arc<Inbox>,
    write: Arc<Inbox>,
    read_limit: Arc<Mutex<Option<Duration>>>,
}

#[cfg(test)]
impl Pipe {
    /// The two ends of a connection.
    pub fn pair() -> (Pipe, Pipe) {
        let (a, b) = (Arc::new(Inbox::default()), Arc::new(Inbox::default()));
        let end = |read: &Arc<Inbox>, write: &Arc<Inbox>| Pipe {
            read: read.clone(),
            write: write.clone(),
            read_limit: Arc::new(Mutex::new(None)),
        };
        (end(&a, &b), end(&b, &a))
    }
}

#[cfg(test)]
impl Read for Pipe {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let limit = *self
            .read_limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.read.take(out, limit)
    }
}

#[cfg(test)]
impl Write for Pipe {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.write.put(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
impl Wire for Pipe {
    fn try_clone(&self) -> std::io::Result<Box<dyn Wire>> {
        Ok(Box::new(self.clone()))
    }

    fn shutdown(&self) -> std::io::Result<()> {
        self.read.end();
        self.write.end();
        Ok(())
    }

    fn set_read_timeout(&self, limit: Option<Duration>) -> std::io::Result<()> {
        *self
            .read_limit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = limit;
        Ok(())
    }

    fn set_write_timeout(&self, _limit: Option<Duration>) -> std::io::Result<()> {
        Ok(())
    }
}
