//! `ComponentRegistry` — XML tag → factory map (PRD §5.4, §6.7, §11.2).
//!
//! The registry is **the** dispatch point from "XML tree of `VNode`s" to
//! "tree of Component instances." Built-in widgets (todo 15+) register
//! at library init; consumer apps register custom widgets at startup;
//! XML template components (todo 27+) register lazily as their files
//! are discovered.
//!
//! ## Resolution order (PRD §5.4)
//!
//! 1. Built-in / Rust-registered component (this module)
//! 2. XML template component (`XmlTemplateLoader`, lands with todo 27)
//! 3. [`DebugBox`] fallback — renders a red outlined rect with the
//!    unknown tag name
//!
//! Step 2 is wired by callers that hold both a `ComponentRegistry` and
//! a `XmlTemplateLoader`; the registry itself only owns steps 1 and 3.
//! Keeping the two layers separate lets tests exercise resolution
//! without dragging the asset loader into the harness.
//!
//! ## Global singleton vs per-test instance
//!
//! [`ComponentRegistry::global()`] returns a lazily-initialised process
//! singleton — convenient for app code and the `register_component!`
//! macro. Unit tests use [`ComponentRegistry::new()`] for isolated
//! state. The global is `Mutex`-protected, not lock-free, because
//! registration only happens at startup (or in tests, never under
//! contention).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::component::Component;
use crate::debug_box::DebugBox;
use crate::vnode::Props;

/// Factory closure type. `Send + Sync` because the *registry* itself
/// can be touched from any thread during startup; the [`Component`]
/// instances the factory returns are not — they stay on the main
/// thread per PRD §6.8.
pub type ComponentFactory = Box<dyn Fn(&Props) -> Box<dyn Component> + Send + Sync>;

/// XML-tag → factory map.
pub struct ComponentRegistry {
    factories: HashMap<String, ComponentFactory>,
    /// Tags whose duplicate-registration warning has already fired.
    /// PRD §6.7 specifies "last-write wins; warn once."
    warned_duplicates: std::collections::HashSet<String>,
}

impl ComponentRegistry {
    /// Construct an empty registry. Tests use this for isolation; the
    /// runtime uses [`global`](Self::global).
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            warned_duplicates: std::collections::HashSet::new(),
        }
    }

    /// Access the process-wide singleton. Lazily initialised. Returns
    /// a [`std::sync::MutexGuard`] so callers naturally serialise on
    /// registration; the lock is contended only at startup.
    pub fn global() -> std::sync::MutexGuard<'static, ComponentRegistry> {
        GLOBAL
            .get_or_init(|| Mutex::new(ComponentRegistry::new()))
            .lock()
            .expect("ComponentRegistry mutex poisoned")
    }

    /// Register `factory` under `tag`. Last write wins, per PRD §6.7;
    /// the first duplicate registration of a given tag logs to stderr
    /// (subsequent duplicates of the same tag are silent so test
    /// fixtures that re-register on every run don't spam).
    pub fn register(&mut self, tag: impl Into<String>, factory: ComponentFactory) {
        let tag = tag.into();
        if self.factories.contains_key(&tag) && self.warned_duplicates.insert(tag.clone()) {
            eprintln!("safi-ui::registry: duplicate registration of <{tag}> — last-write wins");
        }
        self.factories.insert(tag, factory);
    }

    /// Resolve `tag` to a fresh component instance.
    ///
    /// Resolution falls through to [`DebugBox`] for any tag not in the
    /// Rust factory map. The middle layer (XML template lookup) is
    /// handled by the caller when present — see the module docs.
    pub fn resolve(&self, tag: &str, props: &Props) -> Box<dyn Component> {
        if let Some(factory) = self.factories.get(tag) {
            return factory(props);
        }
        Box::new(DebugBox::new(tag))
    }

    /// `true` if a Rust factory is registered under `tag`. Callers
    /// chaining the registry with an XML-template loader use this to
    /// decide which lookup to do first.
    pub fn contains(&self, tag: &str) -> bool {
        self.factories.contains_key(tag)
    }

    /// Number of registered tags. Useful for tests asserting startup
    /// registration count.
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    /// Clear every registered tag. Tests use this to reset the global
    /// between cases when they must use the singleton (e.g. testing
    /// the `register_component!` macro). Production code never calls
    /// this.
    #[doc(hidden)]
    pub fn _clear_for_tests(&mut self) {
        self.factories.clear();
        self.warned_duplicates.clear();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL: OnceLock<Mutex<ComponentRegistry>> = OnceLock::new();

/// Register a Rust component under an XML tag. Convenience over
/// [`ComponentRegistry::global().register`].
///
/// The factory closure receives `&Props` and returns
/// `Box<dyn Component>`. The body usually parses props into a typed
/// constructor:
///
/// ```ignore
/// register_component!("Button", |props| {
///     Box::new(Button {
///         label: props.get_str("label", "Button"),
///         on_press: props.get_str("onPress", ""),
///         bounds: LayoutRect::default(),
///     })
/// });
/// ```
///
/// Note: the macro is `#[macro_export]` so it lives at crate root;
/// the documentation example above uses `ignore` because real usage
/// requires importing both the macro and `PropsExt`.
#[macro_export]
macro_rules! register_component {
    ($tag:expr, $factory:expr) => {{
        let factory: $crate::registry::ComponentFactory = ::std::boxed::Box::new($factory);
        $crate::registry::ComponentRegistry::global().register($tag, factory);
    }};
}
