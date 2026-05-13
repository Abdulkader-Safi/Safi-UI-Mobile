//! window-smoke — the canonical "minimum viable Safi-UI app".
//!
//! Everything you need to ship a Safi-UI app is here: declare a UI tree,
//! point `app_main!` at it. The engine handles SDL3, layout, painting, and
//! lifecycle. Future apps (`hello`, `todo`, `dashboard`) follow this same
//! shape (PRD §17.2).

safi_ui::app_main!(build_ui);

fn build_ui() -> safi_ui::vnode::VNode {
    safi_ui::vnode! {
        <Screen bg="#0f0f1a" width="100%" height="100%">
            <Column gap="16" padding="24" width="100%" height="100%">
                <View bg="#1a1a2e" height="64" />
                <Row flexDirection="row" gap="12" height="120">
                    <View flex="1" height="120" bg="#27ae60" />
                    <View flex="1" height="120" bg="#e74c3c" />
                </Row>
                <View bg="#4f8ef7" flex="1" />
            </Column>
        </Screen>
    }
}
