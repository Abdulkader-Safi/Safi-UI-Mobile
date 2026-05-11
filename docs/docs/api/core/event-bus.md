# `EventBus`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

A main-thread publish/subscribe bus for named events with a cross-thread `post_async` ingress.

```rust
impl EventBus {
    pub fn global() -> &'static EventBus;
    pub fn new() -> Self;

    /// Register a handler. Main-thread only.
    pub fn on(&self, event: &str, handler: impl Fn() + 'static);

    /// Synchronous emit. Main-thread only.
    pub fn emit(&self, event: &str);

    /// Cross-thread safe. Enqueued for dispatch on the next frame.
    pub fn post_async(&self, event: &str);

    /// Drain async queue. Called once per frame by the main loop.
    pub fn drain_async(&self);
}
```

## Usage

```rust
EventBus::global().on("auth.login", || {
    AuthService::login();
});

EventBus::global().emit("auth.login");
EventBus::global().post_async("data.refresh");
```

## XML

Event names are referenced from XML props that take an event-name value:

```xml
<Button label="Sign In" onPress="auth.login" />
<Button label="Save" onPress="{{dynamicAction}}" />
```

## Naming convention

Dot-notation namespaces by feature: `auth.login`, `nav.back`, `modal.open`, `data.refresh`.

## See also

- [State and Events](/guide/concepts/state-and-events)
- [Background-thread state updates](/guide/concepts/state-and-events#background-thread-state-updates)
