use crate::utils::{cargo, deno, from_root, run, ProgressReporter};
use crate::TaskResult;
use anyhow::{bail, Context};
use console::{style, Emoji};
use duct::cmd;
use which::which;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅️ ", "");
static CLIP: Emoji<'_, '_> = Emoji("📎 ", "");
static WARN: Emoji<'_, '_> = Emoji("⚠️ ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
static TEST: Emoji<'_, '_> = Emoji("🧪 ", "");

pub fn test() -> TaskResult<()> {
    let mut progress = ProgressReporter::new(9);
    progress.next_step(LOOKING_GLASS, "Checking prerequisites...");

    let deno_path = which("deno").with_context(|| {
        "Could not find the 'deno' executable. Make sure it is available in your PATH."
    })?;
    progress.report(
        CHECK,
        &format!("Deno found at: {}", deno_path.to_string_lossy()),
    );

    match which("rustup").ok() {
        Some(rustup_path) => {
            let output =
                run(cmd(rustup_path, &["target", "list", "--installed"]).stdout_capture())?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut check_target = |target: &str| match stdout.lines().find(|l| l == &target) {
                Some(_) => {
                    progress.report(CHECK, &format!("Rustup target '{target}' is installed."));
                    Ok(())
                }
                None => {
                    bail!("Rustup target '{}' is not installed.", target);
                }
            };
            check_target("wasm32-unknown-unknown")?;
            check_target("wasm32-wasi")?;
        }
        None => {
            progress.report(WARN, &style("Could not find rustup, so we cannot determine if the appropriate targets are installed.").yellow().to_string());
        }
    }

    progress.next_step(CLIP, "Clippy...");
    run(cargo(["clippy", "--all-features"]).dir(from_root("")))?;

    progress.next_step(CHECK, "Checking formatting...");
    run(cargo(["fmt", "--", "--check"]).dir(from_root("")))?;

    progress.next_step(TRUCK, "Building example protocol...");
    run(cargo(["run"]).dir(from_root("examples/example-protocol")))?;

    progress.next_step(TRUCK, "Building example plugin...");
    run(cargo(["build"]).dir(from_root("examples/example-plugin")))?;

    progress.next_step(TEST, "Running deno tests...");
    run(deno(["test", "--allow-read", "tests.ts"]).dir(from_root("examples/example-deno-runtime")))?;

    progress.next_step(TEST, "Running cargo tests...");
    run(cargo(["test"]).dir(from_root("")))?;

    progress.next_step(TEST, "Running end-to-end wasmer tests...");
    run(cargo(["test"]).dir(from_root("examples/example-rust-wasmer-runtime")))?;

    progress.next_step(TEST, "Running end-to-end wasmer-wasi tests...");
    run(cargo(["test", "--features", "wasi"])
        .dir(from_root("examples/example-rust-wasmer-runtime")))?;

    Ok(())
}
