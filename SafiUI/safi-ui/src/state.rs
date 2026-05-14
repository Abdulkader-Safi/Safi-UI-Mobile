//! `StateStore` — reactive key-value store (PRD §6.12).
//!
//! Backs every `{{key}}` binding in XML. App code writes via
//! [`StateStore::set`]; the renderer pulls via [`StateStore::get`] or
//! through [`PropsExt::resolve_binding`]; widget invalidation flows
//! through per-key subscriptions registered with
//! [`StateStore::subscribe`].
//!
//! ## Threading model (PRD §6.12, §8.4)
//!
//! `set` / `get` / `subscribe` / `unsubscribe` are **main-thread
//! only**. Background work uses [`EventBus::post_async`] to signal
//! completion; the main-thread handler then writes to the store.
//! Cross-thread `set` would race the subscriber callbacks (which can
//! reach into the layout/render world), so the contract is enforced
//! via the `&mut self` receiver — the global singleton guards with a
//! `Mutex` and lock acquisition fails fast under contention rather
//! than corrupting state.
//!
//! [`EventBus::post_async`]: crate::events::EventBus::post_async
//! [`PropsExt::resolve_binding`]: crate::props::PropsExt::resolve_binding

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::props::BindingSource;

/// Opaque handle for a registered subscriber, used to unregister.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubId(u64);

type Subscriber = Box<dyn FnMut(&str) + Send + 'static>;

struct Registration {
    id: SubId,
    callback: Subscriber,
}

pub struct StateStore {
    next_id: u64,
    values: HashMap<String, String>,
    subscribers: HashMap<String, Vec<Registration>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            values: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    /// Process-wide singleton. Lazy `OnceLock<Mutex<…>>` — same
    /// pattern as `EventBus::global` and `ComponentRegistry::global`.
    pub fn global() -> std::sync::MutexGuard<'static, StateStore> {
        GLOBAL
            .get_or_init(|| Mutex::new(StateStore::new()))
            .lock()
            .expect("StateStore mutex poisoned")
    }

    /// Set a value. Triggers every subscriber registered under
    /// `key` synchronously. Returns the previous value, if any.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        let key = key.into();
        let value = value.into();
        let prev = self.values.insert(key.clone(), value.clone());
        if let Some(regs) = self.subscribers.get_mut(&key) {
            for reg in regs {
                (reg.callback)(&value);
            }
        }
        prev
    }

    /// Get a value. Returns `None` if the key has never been set.
    pub fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    /// Returns `true` if `key` has a stored value (regardless of
    /// whether anything has subscribed to it).
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Register a subscriber. Fires every time the key is `set`.
    /// Returns a [`SubId`] for later [`Self::unsubscribe`].
    ///
    /// PRD §6.12: composite bindings (`"Hello {{a}} {{b}}!"`)
    /// register a subscription on **every** key referenced; the
    /// caller is responsible for invoking `subscribe` once per key
    /// from the template (see `crate::props::resolve_composite_with_keys`).
    pub fn subscribe<F>(&mut self, key: impl Into<String>, callback: F) -> SubId
    where
        F: FnMut(&str) + Send + 'static,
    {
        let id = SubId(self.next_id);
        self.next_id += 1;
        self.subscribers
            .entry(key.into())
            .or_default()
            .push(Registration {
                id,
                callback: Box::new(callback),
            });
        id
    }

    /// Unregister a subscriber by id. Returns `true` if a matching
    /// id existed and was removed. Hot-reload (todo 29) and `FlatList`
    /// recycling lean on this so subscriptions don't leak across
    /// reloaded screens.
    pub fn unsubscribe(&mut self, key: &str, id: SubId) -> bool {
        let Some(regs) = self.subscribers.get_mut(key) else {
            return false;
        };
        let before = regs.len();
        regs.retain(|r| r.id != id);
        before != regs.len()
    }

    /// Clear every value + every subscriber. Tests use this to
    /// reset the global between cases; production code never calls
    /// it. Equivalent of `StateStore::new()` for the singleton.
    #[doc(hidden)]
    pub fn _clear_for_tests(&mut self) {
        self.values.clear();
        self.subscribers.clear();
        self.next_id = 1;
    }

    /// Number of distinct keys currently stored.
    pub fn key_count(&self) -> usize {
        self.values.len()
    }

    /// Number of distinct keys with at least one subscriber.
    pub fn subscribed_key_count(&self) -> usize {
        self.subscribers
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .count()
    }

    /// Total registered subscribers across all keys.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.values().map(Vec::len).sum()
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BindingSource for StateStore {
    fn get_binding(&self, key: &str) -> Option<String> {
        self.get(key)
    }
}

static GLOBAL: OnceLock<Mutex<StateStore>> = OnceLock::new();
