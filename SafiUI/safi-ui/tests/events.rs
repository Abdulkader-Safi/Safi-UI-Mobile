//! `EventBus` host tests (todo 24, PRD §6.11, §8.4).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;

use safi_ui::events::EventBus;

#[test]
fn new_bus_is_empty() {
    let bus = EventBus::new();
    assert_eq!(bus.event_count(), 0);
    assert_eq!(bus.handler_count(), 0);
    assert_eq!(bus.handler_count_for("any"), 0);
}

#[test]
fn on_and_emit_round_trip() {
    let mut bus = EventBus::new();
    let calls = Arc::new(AtomicU32::new(0));
    let c = Arc::clone(&calls);
    bus.on("auth.login", move || {
        c.fetch_add(1, Ordering::SeqCst);
    });
    bus.emit("auth.login");
    bus.emit("auth.login");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn emit_unknown_event_is_noop() {
    let mut bus = EventBus::new();
    bus.emit("never.registered"); // must not panic
}

#[test]
fn multiple_handlers_fire_in_registration_order() {
    let mut bus = EventBus::new();
    let log = Arc::new(std::sync::Mutex::new(Vec::<&str>::new()));
    let l1 = Arc::clone(&log);
    let l2 = Arc::clone(&log);
    let l3 = Arc::clone(&log);
    bus.on("nav.tab", move || l1.lock().unwrap().push("first"));
    bus.on("nav.tab", move || l2.lock().unwrap().push("second"));
    bus.on("nav.tab", move || l3.lock().unwrap().push("third"));
    bus.emit("nav.tab");
    assert_eq!(*log.lock().unwrap(), vec!["first", "second", "third"]);
}

#[test]
fn off_unregisters_handler() {
    let mut bus = EventBus::new();
    let calls = Arc::new(AtomicU32::new(0));
    let c = Arc::clone(&calls);
    let id = bus.on("x", move || {
        c.fetch_add(1, Ordering::SeqCst);
    });
    bus.emit("x");
    assert!(bus.off("x", id));
    bus.emit("x");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    // Removing an already-removed id reports false.
    assert!(!bus.off("x", id));
}

#[test]
fn off_with_unknown_name_returns_false() {
    let mut bus = EventBus::new();
    let id = bus.on("real", || {});
    assert!(!bus.off("nonexistent", id));
}

#[test]
fn post_async_from_worker_threads_drains_in_order_on_main() {
    let mut bus = EventBus::new();
    let log = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let l = Arc::clone(&log);
    bus.on("ping", move || {
        l.lock().unwrap().push("ping".into());
    });
    let sender = bus.async_sender();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let tx = sender.clone();
            thread::spawn(move || {
                tx.send("ping".to_string()).unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let drained = bus.drain_async();
    assert_eq!(drained, 8);
    assert_eq!(log.lock().unwrap().len(), 8);
}

#[test]
fn drain_async_with_no_pending_returns_zero() {
    let mut bus = EventBus::new();
    bus.on("idle", || {});
    assert_eq!(bus.drain_async(), 0);
}

#[test]
fn post_async_unknown_event_drains_silently() {
    let mut bus = EventBus::new();
    bus.post_async("no.handlers");
    assert_eq!(bus.drain_async(), 1);
}

#[test]
fn handler_count_tracks_registrations() {
    let mut bus = EventBus::new();
    bus.on("a", || {});
    bus.on("a", || {});
    bus.on("b", || {});
    assert_eq!(bus.event_count(), 2);
    assert_eq!(bus.handler_count(), 3);
    assert_eq!(bus.handler_count_for("a"), 2);
    assert_eq!(bus.handler_count_for("b"), 1);
}
