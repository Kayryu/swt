extern crate cc;
use std::env;

// doc: https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("out_dir: {}", out_dir);
    cc::Build::new().file("foo.c").compile("foo");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=foo");

    println!("cargo:warning=this is debug..... {}", out_dir);
}

// how to rerun the script?
// we have two function: 1. use cargo:rerun-if-changed=PATH 
// 2. use cargo:rerun-if-env-changed=VAR