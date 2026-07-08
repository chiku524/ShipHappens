//! Headless multiplayer smoke binary for CI.
//! Usage: MP_TEST_ROLE=host|join shiphappens_smoke host|join ...

use shiphappens::build_app;

fn main() {
    build_app(true, true).run();
}
