extern crate cc;
use std::process::Command;
use std::{env, path::Path, fs::DirEntry};
use std::fs;

// doc: https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

    println!("cargo:warning=build arch:{}, os:{}, env:{}", arch, os, env);

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("out_dir: {}", out_dir);
    cc::Build::new().file("foo.c").compile("foo");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=foo");

    println!("cargo:warning=this is debug..... {}", out_dir);
}

fn walk_dir<F>(dir: &Path, cb: &F)
where
    F: Fn(&DirEntry),
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, cb);
            } else {
                cb(&entry);
            }
        }
    }
}

fn run_command_with_args<S>(command_name: S, args: &[String])
where
    S: AsRef<std::ffi::OsStr> + Copy,
{
    let mut cmd = Command::new(command_name);
    let _ = cmd.args(args);
    run_command(cmd)
}

fn run_command(mut cmd: Command) {
    eprintln!("running {:?}", cmd);
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to execute [{:?}]: {}", cmd, e);
    });
    if !status.success() {
        panic!("execution failed");
    }
}


// how to rerun the script?
// we have two function: 1. use cargo:rerun-if-changed=PATH 
// 2. use cargo:rerun-if-env-changed=VAR