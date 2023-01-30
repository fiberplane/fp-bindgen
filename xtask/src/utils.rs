use crate::TaskResult;
use anyhow::bail;
use console::{colors_enabled, style, Emoji};
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
        0 => Ok(output),
        _ => {
            bail!("Command invocation did not succeed.")
        }
    }
}

static EMPTY_EMOJI: Emoji<'_, '_> = Emoji("  ", "");

pub struct ProgressReporter {
    cur_step: usize,
    total_steps: usize,
}

impl ProgressReporter {
    pub fn new(total_steps: usize) -> Self {
        Self {
            cur_step: 0,
            total_steps,
        }
    }

    /// Reports status and the current step. Also increases the step count.
    pub fn next_step<'a, 'b>(&mut self, emoji: impl Into<Option<Emoji<'a, 'b>>>, msg: &str) {
        let emoji = emoji.into().unwrap_or(EMPTY_EMOJI);

        self.cur_step += 1;

        println!(
            "{} {}{}",
            style(format!("[{}/{}]", self.cur_step, self.total_steps))
                .bold()
                .dim(),
            emoji,
            msg
        );
    }

    /// Reports status within a step, but doesn't include the step or increase the current step
    pub fn report<'a, 'b>(&mut self, emoji: impl Into<Option<Emoji<'a, 'b>>>, msg: &str) {
        let emoji = emoji.into().unwrap_or(EMPTY_EMOJI);
        println!("      {emoji}{msg}");
    }
}
