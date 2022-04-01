mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[link(name = "libfoo")]
extern "C" {
    fn cadd(a: u32, b: u32) -> u32;
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, test!");
}

#[wasm_bindgen]
pub fn hello(name: String) -> String {
    let c = unsafe {cadd(1, 2) };
    format!("hello {}, add: {}", name, c)
}
