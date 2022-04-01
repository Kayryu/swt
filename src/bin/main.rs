#[link(name="libfoo")]
extern "C" {
    fn cadd(a: u32, b: u32) -> u32;
}

fn main() {
    let c = unsafe { cadd(1, 2) };
    println!("Hello: {}", c);
}