extern crate cc;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    cc::Build::new().file("foo.c").compile("foo");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=foo");
}
