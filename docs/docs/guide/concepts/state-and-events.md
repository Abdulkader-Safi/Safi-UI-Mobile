# State and Events

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned `StateStore` and `EventBus` model.
:::

Safi-UI ships two reactive primitives: a key-value `StateStore` and a named `EventBus`. Both are designed for ergonomic XML data binding without forcing app code into a particular architecture.

## StateStore

A reactive key-value store. Prop bindings use `{{key}}` syntax in XML. `StateStore::set()` calls `DirtyTracker::invalidate_key()`, so only subscribed subtrees rebuild.

```rust
// App code uses the global singleton
StateStore::global().set("user.name", "Safi");
let name = StateStore::global().get("user.name");

// Tests create isolated instances
let store = StateStore::new();

// Subscribe to changes
StateStore::global().subscribe("user.name", |value| {
    println!("Name changed to: {}", value);
});
```

### Binding rules

- Missing key resolves to **empty string** (never an error)
- Bindings are allowed in **any prop**, including `width`, `src`, `onPress`
- Composite bindings supported: `"Hello {{name}}!"`, `"{{first}} {{last}}"`
- Composite bindings register a `DirtyTracker` subscription on **every** key referenced in the template
- Dynamic event bindings supported: `onPress="{{dynamicAction}}"`
- Type coercion: bound values are strings; `PropUtils::parse_*` coerces as normal

### Threading

`set()` and `get()` are **main-thread-only**. The store is not persisted to disk in v1.

## EventBus

A main-thread publish/subscribe bus for named events with a cross-thread `post_async` ingress.

```rust
// Register a handler (main thread)
EventBus::global().on("auth.login", || {
    AuthService::login();
});

// Emit synchronously (main thread only)
EventBus::global().emit("auth.login");

// Safe cross-thread post — dispatched on next frame
EventBus::global().post_async("data.refresh");
```

In XML: `onPress="auth.login"`. Dynamic: `onPress="{{dynamicAction}}"`.

Event names use dot notation by convention.

## Background-thread state updates

Network fetches, file IO, and other background work **cannot** call `StateStore::set` directly. The recommended pattern is to post a marker event from the worker thread and apply the state mutation in its main-thread handler:

```rust
// On a background thread (e.g. an HTTP client task):
let json = http::get("/api/projects").await?;
PENDING_PROJECTS.lock().unwrap().replace(json);
EventBus::global().post_async("projects.fetched");

// Registered once on the main thread at startup:
EventBus::global().on("projects.fetched", || {
    if let Some(json) = PENDING_PROJECTS.lock().unwrap().take() {
        StateStore::global().set("projects.recent", json);
    }
});
```

## Multi-instance and testing

Multi-window is out of scope for v1; one process = one app. A `UIInstance` / `App` handle owns its own `StateStore` and `EventBus`. `::global()` returns the default instance. Unit tests create new `StateStore` and `EventBus` instances per test for clean isolation.

## See also

- [`StateStore` API](/api/core/state-store)
- [`EventBus` API](/api/core/event-bus)
- [Prop Bindings](/guide/authoring/prop-bindings)
