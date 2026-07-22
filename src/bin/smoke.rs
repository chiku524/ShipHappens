//! Headless multiplayer smoke binary for CI.
//! Usage: MP_TEST_ROLE=host|join pudgymon_smoke host|join ...

use pudgymon::build_app;

fn main() {
    build_app(true, true).run();
}
