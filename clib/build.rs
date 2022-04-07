extern crate cc;
use std::fs;
use std::process::Command;
use std::{env, fs::DirEntry, path::Path};

// doc: https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

    println!("cargo:warning=build arch:{}, os:{}, env:{}", arch, os, env);

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("out_dir: {}", out_dir);

    println!("cargo:warning=start to run cc");
    cc::Build::new()
        .include("vendor")
        .file("vendor/foo.c")
        .flag("-Ofast")
        .compile("foo");

    println!("cargo:warning=finished cc");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=foo");

    println!("cargo:warning=this is debug..... {}", out_dir);
}

// how to rerun the script?
// we have two function: 1. use cargo:rerun-if-changed=PATH
// 2. use cargo:rerun-if-env-changed=VAR
