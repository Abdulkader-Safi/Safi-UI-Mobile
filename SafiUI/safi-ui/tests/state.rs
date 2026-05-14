//! `StateStore` host tests (todo 23, PRD §6.12).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use safi_ui::props::{resolve_composite, resolve_composite_with_keys};
use safi_ui::state::StateStore;

#[test]
fn new_store_is_empty() {
    let store = StateStore::new();
    assert_eq!(store.key_count(), 0);
    assert_eq!(store.subscriber_count(), 0);
    assert!(!store.contains("anything"));
}

#[test]
fn set_get_round_trip() {
    let mut store = StateStore::new();
    assert!(store.set("user.name", "Safi").is_none());
    assert_eq!(store.get("user.name").as_deref(), Some("Safi"));
    assert!(store.contains("user.name"));
}

#[test]
fn set_returns_previous_value() {
    let mut store = StateStore::new();
    store.set("k", "first");
    assert_eq!(store.set("k", "second").as_deref(), Some("first"));
    assert_eq!(store.get("k").as_deref(), Some("second"));
}

#[test]
fn get_missing_returns_none() {
    let store = StateStore::new();
    assert!(store.get("missing").is_none());
}

#[test]
fn subscriber_fires_on_set() {
    let mut store = StateStore::new();
    let calls = Arc::new(AtomicU32::new(0));
    let c = Arc::clone(&calls);
    store.subscribe("counter", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    store.set("counter", "1");
    store.set("counter", "2");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn subscriber_receives_new_value() {
    let mut store = StateStore::new();
    let captured = Arc::new(std::sync::Mutex::new(String::new()));
    let cap = Arc::clone(&captured);
    store.subscribe("name", move |v| {
        *cap.lock().unwrap() = v.to_string();
    });
    store.set("name", "Safi");
    assert_eq!(captured.lock().unwrap().as_str(), "Safi");
}

#[test]
fn subscriber_for_other_key_is_not_called() {
    let mut store = StateStore::new();
    let calls = Arc::new(AtomicU32::new(0));
    let c = Arc::clone(&calls);
    store.subscribe("a", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    store.set("b", "1");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn unsubscribe_stops_callbacks() {
    let mut store = StateStore::new();
    let calls = Arc::new(AtomicU32::new(0));
    let c = Arc::clone(&calls);
    let id = store.subscribe("k", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    store.set("k", "1");
    assert!(store.unsubscribe("k", id));
    store.set("k", "2");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    // Removing again returns false.
    assert!(!store.unsubscribe("k", id));
}

#[test]
fn unsubscribe_with_unknown_key_returns_false() {
    let mut store = StateStore::new();
    let id = store.subscribe("real", |_| {});
    assert!(!store.unsubscribe("phantom", id));
}

#[test]
fn binding_source_impl_resolves_via_composite() {
    let mut store = StateStore::new();
    store.set("first", "Abdul");
    store.set("last", "Safi");
    let resolved = resolve_composite("Hello {{first}} {{last}}!", &store);
    assert_eq!(resolved, "Hello Abdul Safi!");
}

#[test]
fn missing_key_resolves_to_empty_string() {
    let store = StateStore::new();
    let resolved = resolve_composite("Hi {{name}}!", &store);
    assert_eq!(resolved, "Hi !");
}

#[test]
fn composite_binding_with_keys_lists_all_referenced() {
    let mut store = StateStore::new();
    store.set("a", "1");
    let (s, keys) = resolve_composite_with_keys("{{a}} + {{b}}", &store);
    assert_eq!(s, "1 + ");
    assert!(keys.contains("a"));
    assert!(keys.contains("b"));
    assert_eq!(keys.len(), 2);
}

#[test]
fn counts_track_state() {
    let mut store = StateStore::new();
    store.set("a", "1");
    store.set("b", "2");
    store.subscribe("a", |_| {});
    store.subscribe("a", |_| {});
    store.subscribe("c", |_| {});
    assert_eq!(store.key_count(), 2);
    assert_eq!(store.subscribed_key_count(), 2);
    assert_eq!(store.subscriber_count(), 3);
}
