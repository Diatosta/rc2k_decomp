//! A machine's outgoing side: frames go into a queue that a thread of its own writes to the
//! connection, in the order they were queued. Sending never waits for the machine, so one that
//! has stopped reading holds up nobody else; a connection whose write fails (or whose queue
//! fills) is cut, which ends its reader.

use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};

use super::*;

/// How many frames may wait for one machine before it is given up as too slow.
const QUEUED_MOST: usize = 4096;

enum Outgoing {
    /// A frame to write.
    Frame(Vec<u8>),
    /// A last frame to write, the connection cut after it.
    Last(Vec<u8>),
}

pub(super) struct Link {
    queue: SyncSender<Outgoing>,
    /// Another handle on the connection, to cut it.
    wire: Box<dyn Wire>,
    /// Told once the writer has ended.
    ended: Receiver<()>,
    /// The cut for a full queue has been logged.
    full_logged: AtomicBool,
}

impl Link {
    /// The queue and its writing thread over `wire`; `None` when the connection cannot be
    /// duplicated.
    pub(super) fn new(wire: Box<dyn Wire>) -> Option<Link> {
        let mut writer = wire.try_clone().ok()?;
        let (queue, frames) = sync_channel::<Outgoing>(QUEUED_MOST);
        let (done, ended) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(next) = frames.recv() {
                let (bytes, last) = match next {
                    Outgoing::Frame(bytes) => (bytes, false),
                    Outgoing::Last(bytes) => (bytes, true),
                };
                if write_frame(&mut writer, &bytes).is_err() || last {
                    break;
                }
            }
            // Whatever ended the writing, the connection ends with it.
            let _ = writer.shutdown();
            let _ = done.send(());
        });
        Some(Link {
            queue,
            wire,
            ended,
            full_logged: AtomicBool::new(false),
        })
    }

    /// `bytes` as a frame after those queued. A machine that has let the queue fill is cut.
    pub(super) fn send(&self, bytes: &[u8]) {
        match self.queue.try_send(Outgoing::Frame(bytes.to_vec())) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                if !self.full_logged.swap(true, Ordering::Relaxed) {
                    crate::program_log::log("net: a machine is too slow to follow and is cut");
                }
                let _ = self.wire.shutdown();
            }
            // The writer has ended and the connection with it.
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    /// `bytes` as the last frame: written after those queued, then the connection is cut.
    pub(super) fn send_last(&self, bytes: &[u8]) {
        if self.queue.try_send(Outgoing::Last(bytes.to_vec())).is_err() {
            let _ = self.wire.shutdown();
        }
    }

    /// Waits up to `limit` for the writer to end (after `send_last`); the connection is cut
    /// at once if it has not.
    pub(super) fn wait_sent(&self, limit: Duration) {
        if self.ended.recv_timeout(limit).is_err() {
            let _ = self.wire.shutdown();
        }
    }
}
