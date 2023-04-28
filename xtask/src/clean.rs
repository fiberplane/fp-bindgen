use crate::utils::{from_root, ProgressReporter};
use crate::TaskResult;
use console::Emoji;
use std::fs::metadata;

static BROOM: Emoji<'_, '_> = Emoji("🧹 ", "");

pub fn clean() -> TaskResult<()> {
    let paths = vec![
        "target",
        "examples/example-plugin/target",
        "examples/example-protocol/bindings",
        "examples/example-rust-wasmer2-runtime/target",
    ];
    let mut progress = ProgressReporter::new(paths.len());

    for path in paths {
        progress.report(BROOM, &format!("Deleting: {}", &path));
        let full_path = from_root(path);
        if let Ok(metadata) = metadata(&full_path) {
            if metadata.is_dir() {
                std::fs::remove_dir_all(full_path)?;
            }
        }
    }

    Ok(())
}
