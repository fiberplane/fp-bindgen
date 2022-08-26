use crate::utils::{cargo, deno, from_root, run};
use crate::TaskResult;
use anyhow::{bail, Context};
use console::{style, Emoji};
use duct::cmd;
use which::which;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅️ ", "");
static WARN: Emoji<'_, '_> = Emoji("⚠️ ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
static TEST: Emoji<'_, '_> = Emoji("🧪 ", "");
static NUM_STEPS: usize = 5;

pub fn test() -> TaskResult<()> {
    let mut cur_step = 0;

    cur_step += 1;
    println!(
        "{} {}Checking prerequisites...",
        style(format!("[{cur_step}/{NUM_STEPS}]")).bold().dim(),
        LOOKING_GLASS
    );

    let deno_path = which("deno").with_context(|| {
        format!("Could not find the 'deno' executable. Make sure it is available in your PATH.")
    })?;
    println!(
        "{} {}Deno found at: {}",
        style("     ").bold().dim(),
        CHECK,
        deno_path.to_string_lossy()
    );

    match which("rustup").ok() {
        Some(rustup_path) => {
            let output =
                run(cmd(rustup_path, &["target", "list", "--installed"]).stdout_capture())?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            match stdout.lines().find(|l| l == &"wasm32-unknown-unknown") {
                Some(_) => {
                    println!(
                        "{} {}Rustup target 'wasm32-unknown-unknown' is installed.",
                        style("     ").bold().dim(),
                        CHECK,
                    );
                }
                None => {
                    bail!("Rustup target 'wasm32-unknown-unknown' is not installed.");
                }
            }
        }
        None => {
            println!(
                "{} {}{}",
                style("     ").bold().dim(),
                WARN,
                style("Could not find rustup, so we cannot determine if the appropriate targets are installed.").yellow()
            );
        }
    }

    cur_step += 1;
    println!(
        "{} {}Building example protocol...",
        style(format!("[{cur_step}/{NUM_STEPS}]")).bold().dim(),
        TRUCK
    );
    run(cargo(&["run"]).dir(from_root("examples/example-protocol")))?;

    cur_step += 1;
    println!(
        "{} {}Building example plugin...",
        style(format!("[{cur_step}/{NUM_STEPS}]")).bold().dim(),
        TRUCK
    );
    run(cargo(&["build"]).dir(from_root("examples/example-plugin")))?;

    cur_step += 1;
    println!(
        "{} {}Running Deno tests...",
        style(format!("[{cur_step}/{NUM_STEPS}]")).bold().dim(),
        TEST
    );
    run(
        deno(&["test", "--allow-read", "tests.ts"]).dir(from_root("examples/example-deno-runtime"))
    )?;

    cur_step += 1;
    println!(
        "{} {}Run end-to-end wasmer runtime tests...",
        style(format!("[{cur_step}/{NUM_STEPS}]")).bold().dim(),
        TEST
    );
    run(cargo(&["test"]).dir(from_root("examples/example-rust-runtime")))?;

    Ok(())
}
