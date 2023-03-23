mod spec;
#[cfg(test)]
mod test;
mod wasi_spec;

use std::sync::Mutex;

pub static GLOBAL_STATE: Mutex<u32> = Mutex::new(0);

fn main() {
    println!("Hello, world!");
}
