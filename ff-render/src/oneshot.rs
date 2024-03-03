//! Sync-to-sync/async oneshot channel.
//!
//! New implementation to avoid pulling in all of tokio or similar implementations.

use std::{
    future::Future,
    sync::{Arc, Condvar, Mutex},
    task::{Poll, Waker},
};

/// Create a new oneshot channel.
///
/// The sender can send synchronously; no error is given if the receiver has hung up.
/// The sender can check
pub fn new<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Sync {
        state: Mutex::new(State::Idle),
        cv: Condvar::new(),
    });
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver { shared: shared },
    )
}

/// Sender side of a oneshot channel.
pub struct Sender<T> {
    shared: Arc<Sync<T>>,
}

impl<T> Sender<T> {
    /// Returns true if the receiver has hung up.
    pub fn is_cancelled(&self) -> bool {
        let g = match self.shared.state.lock() {
            Err(_) => return true,
            Ok(v) => v,
        };
        if let State::ReceiverDropped = *g {
            true
        } else {
            false
        }
    }

    /// Sends the provided value.
    pub fn send(self, value: T) {
        let mut g = match self.shared.state.lock() {
            Err(_) => return,
            Ok(v) => v,
        };
        g.done(value);
        self.shared.cv.notify_one();
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut g = match self.shared.state.lock() {
            Err(_) => return,
            Ok(v) => v,
        };
        g.drop_send();
        // If there was a sync receiver, notify it:
        self.shared.cv.notify_one();
    }
}

pub struct Receiver<T> {
    shared: Arc<Sync<T>>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut g = match self.shared.state.lock() {
            Err(_) => return,
            Ok(v) => v,
        };
        g.drop_recv();
    }
}

impl<T> Receiver<T> {
    /// Synchronously receive a value.
    /// Returns an error if the sender hung up prematurely or another error occurred.
    ///
    /// This is a "consuming" method; there are no retried.
    pub fn recv(self) -> Result<T, &'static str> {
        let mut g = match self.shared.state.lock() {
            Err(_) => return Err("lock corrupted"),
            Ok(v) => v,
        };
        loop {
            if let Some(v) = g.take() {
                return Ok(v);
            }
            if let State::SenderDropped = *g {
                return Err("sender hung up");
            }
            // Wait for condition variable to update.
            g = match self.shared.cv.wait(g) {
                Ok(g) => g,
                Err(_) => return Err("lock poisoned"),
            }
        }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, &'static str>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut g = match self.shared.state.lock() {
            Err(_) => return Poll::Ready(Err("lock poisoned")),
            Ok(g) => g,
        };
        if let Some(v) = g.take() {
            return Poll::Ready(Ok(v));
        }
        if let State::SenderDropped = *g {
            return Poll::Ready(Err("sender hung up"));
        }

        // Register ourselves:
        *g = State::WaitingForValue(cx.waker().clone());
        Poll::Pending
    }
}

enum State<T> {
    Idle,
    WaitingForValue(Waker),
    Ready(T),
    ReceiverDropped,
    SenderDropped,
}

impl<T> State<T> {
    fn take(&mut self) -> Option<T> {
        let mut alt = State::ReceiverDropped;
        std::mem::swap(self, &mut alt);
        if let State::Ready(v) = alt {
            Some(v)
        } else {
            std::mem::swap(self, &mut alt);
            None
        }
    }
    fn done(&mut self, value: T) {
        let mut alt = State::Ready(value);
        std::mem::swap(self, &mut alt);
        if let State::WaitingForValue(w) = alt {
            w.wake();
        }
    }

    fn drop_send(&mut self) {
        match self {
            State::Ready(_) | State::ReceiverDropped => return,
            _ => (),
        };
        let mut alt = State::SenderDropped;
        std::mem::swap(self, &mut alt);
        if let State::WaitingForValue(w) = alt {
            w.wake();
        }
    }

    fn drop_recv(&mut self) {
        *self = State::ReceiverDropped;
    }
}

struct Sync<T> {
    state: Mutex<State<T>>,
    cv: Condvar,
}
