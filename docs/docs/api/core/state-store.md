# `StateStore`

:::tip Status: ✅ Shipped (todo 23)
Reactive key-value store driving `{{binding}}` props. `App::run` resolves bindings from `StateStore::global()` every frame; `set(k, v)` fires every registered subscriber for that key.
:::

The backing store for every `{{key}}` binding in XML. App code writes Rust-side state; the renderer pulls via [`BindingSource`] when walking the tree.

[`BindingSource`]: ./prop-utils

## Definition

```rust
use safi_ui::state::{StateStore, SubId};

impl StateStore {
    pub fn new() -> Self;
    pub fn global() -> MutexGuard<'static, Self>;

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String>;
    pub fn get(&self, key: &str) -> Option<String>;
    pub fn contains(&self, key: &str) -> bool;

    pub fn subscribe(&mut self, key: impl Into<String>,
                     callback: impl FnMut(&str) + Send + 'static) -> SubId;
    pub fn unsubscribe(&mut self, key: &str, id: SubId) -> bool;

    pub fn key_count(&self) -> usize;
    pub fn subscribed_key_count(&self) -> usize;
    pub fn subscriber_count(&self) -> usize;
}
```

## Threading

`set` / `get` / `subscribe` / `unsubscribe` are **main thread only** (PRD §6.12). Background threads signal completion via `EventBus::post_async`; the main-thread handler reads work product and calls `set`:

```rust
use safi_ui::events::EventBus;
use safi_ui::state::StateStore;

// Worker thread:
let tx = EventBus::global().async_sender();
std::thread::spawn(move || {
    let projects = fetch_http_blocking();
    PENDING.lock().unwrap().replace(projects);
    tx.send("projects.fetched".into()).unwrap();
});

// Main thread, registered once at startup:
EventBus::global().on("projects.fetched", || {
    if let Some(json) = PENDING.lock().unwrap().take() {
        StateStore::global().set("projects.recent", json);
    }
});
```

## Reactive bindings

Any prop value containing `{{key}}` resolves against the store at every frame's build pass:

```xml
<Screen bg="#0f0f1a">
  <Heading level="2" color="#fff">Welcome, {{user.name}}!</Heading>
  <Text color="#aaa">You have {{notifications.count}} unread.</Text>
</Screen>
```

Composite bindings (`"Hello {{first}} {{last}}!"`) substitute every key referenced; missing keys collapse to empty string (never an error, never a panic).

App-side state-change → re-render is driven by the frame loop running every iteration; the retained-mode `DirtyTracker` integration that avoids re-walking the full tree on every change lands in a follow-up.

## Manual subscriptions

For Rust-side reactions (not bound to XML), subscribe directly:

```rust
let id = StateStore::global().subscribe("cart.total", |v| {
    println!("Cart total is now {v}");
});
// later:
StateStore::global().unsubscribe("cart.total", id);
```

`subscribe` returns a [`SubId`] for clean unregistration — important during hot-reload (todo 29) and `FlatList` recycling so subscribers don't leak across reloaded screens.

## Multi-instance

Multi-window is out of scope for v1. `StateStore::global()` returns the process singleton owned by the active `App`. Tests construct fresh instances with `StateStore::new()`.

## Persistence

The store is **not** persisted to disk in v1. App code that needs persistence should serialize on its own schedule (e.g., on `WillEnterBackground`).

## See also

- [`EventBus`](/api/core/event-bus) — async ingress + Button-tap dispatch
- [`PropUtils`](/api/core/prop-utils) — `{{key}}` template syntax + `BindingSource` trait
- [Prop Bindings](/guide/authoring/prop-bindings)
- [PRD §6.12](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md)
