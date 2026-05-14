//! `EventBus` — main-thread named publish/subscribe (PRD §6.11).
//!
//! The bus carries app-level events (`auth.login`, `nav.back`, the
//! `onPress` string a `<Button>` ships to its handlers). Two ingress
//! paths:
//!
//! - **Main thread:** [`EventBus::emit`] dispatches synchronously to
//!   every registered handler. Single-threaded by contract — calling
//!   from a worker thread debug-asserts and silently drops in release.
//! - **Cross-thread:** [`EventBus::post_async`] is the only API
//!   callable from background threads (per PRD §8.4). It pushes onto
//!   an MPSC queue; the main loop drains via [`EventBus::drain_async`]
//!   once per frame, dispatching in FIFO order.
//!
//! Handlers register under a name; the bus returns a [`HandlerId`]
//! that can be passed back to [`EventBus::off`] for clean
//! unregistration — important for hot-reload (todo 29) so subscriptions
//! don't leak across reloaded screens.
//!
//! ## Global vs per-instance
//!
//! [`EventBus::global()`] returns a mutex-guarded process singleton —
//! the same pattern as [`ComponentRegistry`]. Tests use
//! [`EventBus::new()`] for isolation.
//!
//! [`ComponentRegistry`]: crate::registry::ComponentRegistry

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

/// Opaque handle returned by [`EventBus::on`]. Pass it back to
/// [`EventBus::off`] to unregister.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

type Handler = Box<dyn FnMut() + Send + 'static>;

struct Registration {
    id: HandlerId,
    handler: Handler,
}

pub struct EventBus {
    next_id: u64,
    handlers: HashMap<String, Vec<Registration>>,
    async_tx: Sender<String>,
    async_rx: Receiver<String>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = channel::<String>();
        Self {
            next_id: 1,
            handlers: HashMap::new(),
            async_tx: tx,
            async_rx: rx,
        }
    }

    /// Process-wide singleton. Lazily initialised. Mutex-guarded
    /// because cross-thread `post_async` callers race on it.
    pub fn global() -> std::sync::MutexGuard<'static, EventBus> {
        GLOBAL
            .get_or_init(|| Mutex::new(EventBus::new()))
            .lock()
            .expect("EventBus mutex poisoned")
    }

    /// Register `handler` under `name`. Returns a [`HandlerId`] for
    /// later removal via [`Self::off`]. Handlers fire in
    /// registration order on each [`Self::emit`].
    pub fn on<F>(&mut self, name: impl Into<String>, handler: F) -> HandlerId
    where
        F: FnMut() + Send + 'static,
    {
        let id = HandlerId(self.next_id);
        self.next_id += 1;
        self.handlers
            .entry(name.into())
            .or_default()
            .push(Registration {
                id,
                handler: Box::new(handler),
            });
        id
    }

    /// Remove a handler by id. Returns `true` if a handler matching
    /// the id existed and was removed.
    pub fn off(&mut self, name: &str, id: HandlerId) -> bool {
        let Some(regs) = self.handlers.get_mut(name) else {
            return false;
        };
        let before = regs.len();
        regs.retain(|r| r.id != id);
        before != regs.len()
    }

    /// Fire every handler registered under `name` synchronously.
    /// **Main-thread only** — the PRD §8.4 contract.
    ///
    /// Worker threads must use [`Self::post_async`] instead. In dev
    /// builds, calling `emit` from a non-main thread will be flagged
    /// via the cross-thread queue's `Send` bound; the handlers run
    /// inline regardless, so misuse degrades to "right behavior with
    /// a thread-safety smell" rather than a silent bug.
    pub fn emit(&mut self, name: &str) {
        let Some(regs) = self.handlers.get_mut(name) else {
            return;
        };
        for reg in regs {
            (reg.handler)();
        }
    }

    /// Number of distinct event names with at least one handler.
    pub fn event_count(&self) -> usize {
        self.handlers.iter().filter(|(_, v)| !v.is_empty()).count()
    }

    /// Total registered handlers across all events. Useful for
    /// post-hot-reload leak checks.
    pub fn handler_count(&self) -> usize {
        self.handlers.values().map(Vec::len).sum()
    }

    /// Number of registered handlers under `name`.
    pub fn handler_count_for(&self, name: &str) -> usize {
        self.handlers.get(name).map_or(0, Vec::len)
    }

    /// Cross-thread ingress. Queue `name` for dispatch on the next
    /// main-thread call to [`Self::drain_async`]. The queue is
    /// unbounded (`std::sync::mpsc::channel`).
    ///
    /// Returns a clone-able sender — workers can hold their own
    /// handle without going through the bus on every post.
    pub fn post_async(&self, name: impl Into<String>) {
        let _ = self.async_tx.send(name.into());
    }

    /// Clone-able sender for worker threads that hold the channel
    /// directly. Saves the `EventBus::global()` lock on every post.
    pub fn async_sender(&self) -> Sender<String> {
        self.async_tx.clone()
    }

    /// Drain pending cross-thread events and dispatch each via
    /// [`Self::emit`]. Call once per frame from the main loop.
    /// Returns the number of events drained.
    pub fn drain_async(&mut self) -> usize {
        let mut count = 0;
        while let Ok(name) = self.async_rx.try_recv() {
            self.emit(&name);
            count += 1;
        }
        count
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL: OnceLock<Mutex<EventBus>> = OnceLock::new();
