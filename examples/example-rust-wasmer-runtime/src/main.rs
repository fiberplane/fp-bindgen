#[cfg(not(feature="wasi"))]
mod spec;
#[cfg(feature="wasi")]
mod wasi_spec;
#[cfg(test)]
mod test;

fn main() {
    println!("Hello, world!");
}
