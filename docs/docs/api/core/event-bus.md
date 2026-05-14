# `EventBus`

:::tip Status: ✅ Shipped (todo 24)
Main-thread publish/subscribe + cross-thread `post_async` ingress per PRD §6.11 / §8.4. Button taps dispatch automatically via `App::run`'s hit-test pump.
:::

A named publish/subscribe bus. App logic (Rust) registers handlers under event names; XML wires `<Button onPress="event.name">` to trigger them on tap.

## Definition

```rust
use safi_ui::events::{EventBus, HandlerId};

pub struct EventBus { /* … */ }

impl EventBus {
    pub fn new() -> Self;
    pub fn global() -> MutexGuard<'static, Self>;

    /// Register handler. Returns id for later `off`. Main thread.
    pub fn on(&mut self, name: impl Into<String>, handler: impl FnMut() + Send + 'static) -> HandlerId;

    /// Unregister by id. Returns true if removed.
    pub fn off(&mut self, name: &str, id: HandlerId) -> bool;

    /// Synchronous dispatch. Main thread only.
    pub fn emit(&mut self, name: &str);

    /// Cross-thread ingress. Queues for next drain_async.
    pub fn post_async(&self, name: impl Into<String>);

    /// Clone-able sender — workers hold the channel directly.
    pub fn async_sender(&self) -> std::sync::mpsc::Sender<String>;

    /// Drain queue and dispatch in FIFO order. Frame loop calls
    /// once per frame.
    pub fn drain_async(&mut self) -> usize;

    pub fn event_count(&self) -> usize;
    pub fn handler_count(&self) -> usize;
    pub fn handler_count_for(&self, name: &str) -> usize;
}
```

## Threading contract

- `on` / `off` / `emit` / `drain_async` — **main thread only**.
- `post_async` / `async_sender` — **any thread**. Workers hold a clone-able `Sender<String>` and post without contending on the bus's mutex on every send.

The frame loop in `App::run` calls `EventBus::global().drain_async()` once per frame — every `post_async` from the previous frame fires synchronously on the main thread before the next render pass.

## Usage

```rust
use safi_ui::events::EventBus;

// At app startup (main thread):
{
    let mut bus = EventBus::global();
    bus.on("auth.login", || {
        // Rust app logic — fire HTTP request, navigate, etc.
    });
    bus.on("nav.back", || {
        // ...
    });
}
```

In XML:

```xml
<Button label="Sign In" onPress="auth.login" />
<Button label="Save" onPress="{{dynamicAction}}" />
```

When the user taps the button, `App::run` hit-tests the touch against the laid-out VNode tree, finds the deepest node with an `onPress` prop, and calls `EventBus::global().emit(name)`.

## Background-thread pattern (PRD §6.12)

State mutations must happen on the main thread. Workers signal completion via `post_async`:

```rust
use safi_ui::events::EventBus;

// Worker thread:
let sender = EventBus::global().async_sender();
std::thread::spawn(move || {
    let _result = blocking_http_request();
    PENDING.lock().unwrap().replace(_result);
    sender.send("projects.fetched".into()).unwrap();
});

// Main thread, registered once at startup:
{
    let mut bus = EventBus::global();
    bus.on("projects.fetched", || {
        if let Some(json) = PENDING.lock().unwrap().take() {
            // Update StateStore (todo 23) here — main thread.
        }
    });
}
```

## Naming convention

Dot-notation namespaces by feature: `auth.login`, `nav.back`, `modal.open`, `data.refresh`. The bus itself doesn't enforce any structure — names are arbitrary strings.

## See also

- [`StateStore`](/api/core/state-store) — reactive key-value store (todo 23)
- [PRD §6.11 / §8.4](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md)
