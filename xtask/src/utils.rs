use crate::TaskResult;
use anyhow::bail;
use console::colors_enabled;
use duct::{cmd, Expression};
use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

pub fn cargo(args: impl IntoIterator<Item = impl Into<OsString>>) -> Expression {
    let mut args: VecDeque<OsString> = args.into_iter().map(|os| os.into()).collect();

    if colors_enabled() {
        args.push_front("always".into());
        args.push_front("--color".into());
    }

    cmd("cargo", &args)
}

pub fn deno(args: impl IntoIterator<Item = impl Into<OsString>>) -> Expression {
    let args: VecDeque<OsString> = args.into_iter().map(|os| os.into()).collect();
    cmd("deno", &args)
}

pub fn from_root(path: impl Into<PathBuf>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(path.into())
}

pub fn run(expr: Expression) -> TaskResult<Output> {
    let output = expr.unchecked().run()?;
    let code = output.status.code().unwrap();
    match code {
        0 => Ok(output.clone()),
        _ => {
            bail!("Command invocation did not succeed.")
        }
    }
}
