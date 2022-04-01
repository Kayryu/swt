mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern "C" {
    fn cadd(a: u32, b: u32) -> u32;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, test!");
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn hello(name: String) -> String {
    let len = name.len() as u32;
    let c = unsafe { cadd(len, len) };
    format!("hello {}. The libc result: {}", name, c)
}
