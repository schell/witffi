//! Build script that delegates to `xtask` to generate FFI scaffolding,
//! C headers, Kotlin bindings, and Swift bindings from the eip681 WIT
//! definition.

use std::path::Path;

fn main() {
    let workspace_root = Path::new(env!("CARGO_WORKSPACE_DIR"));
    println!(
        "cargo::rerun-if-changed={}",
        workspace_root.join("wit/eip681.wit").display()
    );
    xtask::generate(workspace_root).expect("xtask binding generation failed");
}
