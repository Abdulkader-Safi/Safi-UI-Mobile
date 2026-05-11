# `StateStore`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

A reactive key-value store. **Main-thread only.** Each `set()` calls `DirtyTracker::invalidate_key()`, so only subscribed subtrees rebuild.

```rust
impl StateStore {
    pub fn global() -> &'static StateStore;
    pub fn new() -> Self;

    pub fn set(&self, key: &str, value: impl Into<String>);
    pub fn get(&self, key: &str) -> String;

    pub fn subscribe(
        &self,
        key: &str,
        callback: impl Fn(&str) + 'static,
    );
}
```

## Usage

```rust
StateStore::global().set("user.name", "Safi");
let name = StateStore::global().get("user.name");

StateStore::global().subscribe("user.name", |value| {
    println!("Name changed to: {}", value);
});

// Tests
let store = StateStore::new();
```

## XML binding

In XML, use `{{key}}` to reference a value:

```xml
<Text>{{user.name}}</Text>
```

See [Prop Bindings](/guide/authoring/prop-bindings).

## Threading

`set()` and `get()` are **main-thread-only**. Background threads must use [`EventBus::post_async`](/api/core/event-bus) to trigger main-thread state mutations. See the [recommended pattern](/guide/concepts/state-and-events#background-thread-state-updates).

## Multi-instance

Multi-window is out of scope for v1. `StateStore::global()` returns the default instance owned by the active `App`. Tests construct fresh instances with `StateStore::new()`.

## Persistence

The store is **not** persisted to disk in v1. App code that needs persistence should serialize on its own schedule (e.g., on `WillEnterBackground`).
