extern crate cc;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::{env, path::Path, fs::DirEntry};
use std::fs;

// doc: https://doc.rust-lang.org/cargo/reference/build-scripts.html

const RING_INCLUDES: &[&str] =
    &[
      "foo.h",
    ];

const RING_BUILD_FILE: &[&str] = &["build.rs"];

const RING_SRCS: &[(&[&str], &str)] = &[
    (&[], "foo.c"),

];

fn main() {
    ring_build_rs_main();
}

fn c_flags(target: &Target) -> &'static [&'static str] {
    if target.env != MSVC {
        static NON_MSVC_FLAGS: &[&str] = &[
            "-std=c1x", // GCC 4.6 requires "c1x" instead of "c11"
            "-Wbad-function-cast",
            "-Wnested-externs",
            "-Wstrict-prototypes",
        ];
        NON_MSVC_FLAGS
    } else {
        &[]
    }
}

fn cpp_flags(target: &Target) -> &'static [&'static str] {
    if target.env != MSVC {
        static NON_MSVC_FLAGS: &[&str] = &[
            "-pedantic",
            "-pedantic-errors",
            "-Wall",
            "-Wextra",
            "-Wcast-align",
            "-Wcast-qual",
            "-Wconversion",
            "-Wenum-compare",
            "-Wfloat-equal",
            "-Wformat=2",
            "-Winline",
            "-Winvalid-pch",
            "-Wmissing-field-initializers",
            "-Wmissing-include-dirs",
            "-Wredundant-decls",
            "-Wshadow",
            "-Wsign-compare",
            "-Wsign-conversion",
            "-Wundef",
            "-Wuninitialized",
            "-Wwrite-strings",
            "-fno-strict-aliasing",
            "-fvisibility=hidden",
        ];
        NON_MSVC_FLAGS
    } else {
        static MSVC_FLAGS: &[&str] = &[
            "/GS",   // Buffer security checks.
            "/Gy",   // Enable function-level linking.
            "/EHsc", // C++ exceptions only, only in C++.
            "/GR-",  // Disable RTTI.
            "/Zc:wchar_t",
            "/Zc:forScope",
            "/Zc:inline",
            "/Zc:rvalueCast",
            // Warnings.
            "/sdl",
            "/Wall",
            "/wd4127", // C4127: conditional expression is constant
            "/wd4464", // C4464: relative include path contains '..'
            "/wd4514", // C4514: <name>: unreferenced inline function has be
            "/wd4710", // C4710: function not inlined
            "/wd4711", // C4711: function 'function' selected for inline expansion
            "/wd4820", // C4820: <struct>: <n> bytes padding added after <name>
            "/wd5045", /* C5045: Compiler will insert Spectre mitigation for memory load if
                        * /Qspectre switch specified */
        ];
        MSVC_FLAGS
    }
}

const LD_FLAGS: &[&str] = &[];

const MSVC: &str = "msvc";
const MSVC_OBJ_OPT: &str = "/Fo";
const MSVC_OBJ_EXT: &str = "obj";

fn ring_build_rs_main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let (obj_ext, obj_opt) = if env == MSVC {
        (MSVC_OBJ_EXT, MSVC_OBJ_OPT)
    } else {
        ("o", "-o")
    };

    let is_git = std::fs::metadata(".git").is_ok();

    // Published builds are always release builds.
    let is_debug = is_git && env::var("DEBUG").unwrap() != "false";

    let target = Target {
        arch,
        os,
        env,
        obj_ext,
        obj_opt,
        is_git,
        is_debug,
    };

    build_c_code(&target, &out_dir);
    check_all_files_tracked()
}

struct Target {
    arch: String,
    os: String,
    env: String,
    obj_ext: &'static str,
    obj_opt: &'static str,
    is_git: bool,
    is_debug: bool,
}

fn build_c_code(target: &Target, out_dir: &Path) {
    #[cfg(not(feature = "wasm32_c"))]
    {
        if &target.arch == "wasm32" {
            println!("cargo:warning=build for wasm");
            return;
        }
    }

    let includes_modified = RING_INCLUDES
        .iter()
        .chain(RING_BUILD_FILE.iter())
        .map(|f| file_modified(Path::new(*f)))
        .max()
        .unwrap();

    let warnings_are_errors = target.is_git;

    let asm_srcs= Vec::new();

    let core_srcs = sources_for_arch(&target.arch)
        .into_iter()
        .filter(|p| !is_perlasm(&p))
        .collect::<Vec<_>>();

    let libs = [
        ("ring-core", &core_srcs[..], &asm_srcs[..]),
    ];

    // XXX: Ideally, ring-test would only be built for `cargo test`, but Cargo
    // can't do that yet.
    libs.iter().for_each(|&(lib_name, srcs, additional_srcs)| {
        build_library(
            &target,
            &out_dir,
            lib_name,
            srcs,
            additional_srcs,
            warnings_are_errors,
            includes_modified,
        )
    });

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_str().expect("Invalid path")
    );
}

fn build_library(
    target: &Target,
    out_dir: &Path,
    lib_name: &str,
    srcs: &[PathBuf],
    additional_srcs: &[PathBuf],
    warnings_are_errors: bool,
    includes_modified: SystemTime,
) {
    // Compile all the (dirty) source files into object files.
    let objs = additional_srcs
        .iter()
        .chain(srcs.iter())
        .filter(|f| &target.env != "msvc" || f.extension().unwrap().to_str().unwrap() != "S")
        .map(|f| compile(f, target, warnings_are_errors, out_dir, includes_modified))
        .collect::<Vec<_>>();

    // Rebuild the library if necessary.
    let lib_path = PathBuf::from(out_dir).join(format!("lib{}.a", lib_name));

    if objs
        .iter()
        .map(Path::new)
        .any(|p| need_run(&p, &lib_path, includes_modified))
    {
        let mut c = cc::Build::new();

        for f in LD_FLAGS {
            let _ = c.flag(&f);
        }
        match target.os.as_str() {
            "macos" => {
                let _ = c.flag("-fPIC");
                let _ = c.flag("-Wl,-dead_strip");
            }
            _ => {
                let _ = c.flag("-Wl,--gc-sections");
            }
        }
        for o in objs {
            let _ = c.object(o);
        }

        // Handled below.
        let _ = c.cargo_metadata(false);

        c.compile(
            lib_path
                .file_name()
                .and_then(|f| f.to_str())
                .expect("No filename"),
        );
    }

    // Link the library. This works even when the library doesn't need to be
    // rebuilt.
    println!("cargo:rustc-link-lib=static={}", lib_name);
}

fn compile(
    p: &Path,
    target: &Target,
    warnings_are_errors: bool,
    out_dir: &Path,
    includes_modified: SystemTime,
) -> String {
    let ext = p.extension().unwrap().to_str().unwrap();
    if ext == "obj" {
        p.to_str().expect("Invalid path").into()
    } else {
        let mut out_path = out_dir.join(p.file_name().unwrap());
        assert!(out_path.set_extension(target.obj_ext));
        if need_run(&p, &out_path, includes_modified) {
            let cmd = cc(p, ext, target, warnings_are_errors, &out_path);
            run_command(cmd);
        }
        out_path.to_str().expect("Invalid path").into()
    }
}

fn cc(
    file: &Path,
    ext: &str,
    target: &Target,
    warnings_are_errors: bool,
    out_dir: &Path,
) -> Command {
    let is_musl = target.env.starts_with("musl");

    let mut c = cc::Build::new();
    let _ = c.include("include");
    match ext {
        "c" => {
            for f in c_flags(target) {
                let _ = c.flag(f);
            }
        }
        "S" => (),
        e => panic!("Unsupported file extension: {:?}", e),
    };
    for f in cpp_flags(target) {
        let _ = c.flag(&f);
    }
    if target.os != "none"
        && target.os != "redox"
        && target.os != "windows"
        && target.arch != "wasm32"
    {
        let _ = c.flag("-fstack-protector");
    }

    match (target.os.as_str(), target.env.as_str()) {
        // ``-gfull`` is required for Darwin's |-dead_strip|.
        ("macos", _) => {
            let _ = c.flag("-gfull");
        }
        (_, "msvc") => (),
        _ => {
            let _ = c.flag("-g3");
        }
    };
    if !target.is_debug {
        let _ = c.define("NDEBUG", None);
    }

    if &target.env == "msvc" {
        if std::env::var("OPT_LEVEL").unwrap() == "0" {
            let _ = c.flag("/Od"); // Disable optimization for debug builds.
                                   // run-time checking: (s)tack frame, (u)ninitialized variables
            let _ = c.flag("/RTCsu");
        } else {
            let _ = c.flag("/Ox"); // Enable full optimization.
        }
    }

    // Allow cross-compiling without a target sysroot for these targets.
    //
    // poly1305_vec.c requires <emmintrin.h> which requires <stdlib.h>.
    if (target.arch == "wasm32" && target.os == "unknown")
        || (target.os == "linux" && is_musl && target.arch != "x86_64")
    {
        if let Ok(compiler) = c.try_get_compiler() {
            // TODO: Expand this to non-clang compilers in 0.17.0 if practical.
            if compiler.is_like_clang() {
                let _ = c.flag("-nostdlibinc");
                let _ = c.define("GFp_NOSTDLIBINC", "1");
            }
        }
    }

    if warnings_are_errors {
        let flag = if &target.env != "msvc" {
            "-Werror"
        } else {
            "/WX"
        };
        let _ = c.flag(flag);
    }
    if is_musl {
        // Some platforms enable _FORTIFY_SOURCE by default, but musl
        // libc doesn't support it yet. See
        // http://wiki.musl-libc.org/wiki/Future_Ideas#Fortify
        // http://www.openwall.com/lists/musl/2015/02/04/3
        // http://www.openwall.com/lists/musl/2015/06/17/1
        let _ = c.flag("-U_FORTIFY_SOURCE");
    }

    let mut c = c.get_compiler().to_command();
    let _ = c
        .arg("-c")
        .arg(format!(
            "{}{}",
            target.obj_opt,
            out_dir.to_str().expect("Invalid path")
        ))
        .arg(file);
    c
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

fn sources_for_arch(arch: &str) -> Vec<PathBuf> {
    RING_SRCS
        .iter()
        .filter(|&&(archs, _)| archs.is_empty() || archs.contains(&arch))
        .map(|&(_, p)| PathBuf::from(p))
        .collect::<Vec<_>>()
}

fn is_perlasm(path: &PathBuf) -> bool {
    path.extension().unwrap().to_str().unwrap() == "pl"
}

fn need_run(source: &Path, target: &Path, includes_modified: SystemTime) -> bool {
    let s_modified = file_modified(source);
    if let Ok(target_metadata) = std::fs::metadata(target) {
        let target_modified = target_metadata.modified().unwrap();
        s_modified >= target_modified || includes_modified >= target_modified
    } else {
        // On error fetching metadata for the target file, assume the target
        // doesn't exist.
        true
    }
}

fn file_modified(path: &Path) -> SystemTime {
    let path = Path::new(path);
    let path_as_str = format!("{:?}", path);
    std::fs::metadata(path)
        .expect(&path_as_str)
        .modified()
        .expect("nah")
}

fn check_all_files_tracked() {
    for path in &["crypto", "include", "third_party/fiat"] {
        walk_dir(&PathBuf::from(path), &is_tracked);
    }
}

fn is_tracked(file: &DirEntry) {
    let p = file.path();
    let cmp = |f| p == PathBuf::from(f);
    let tracked = match p.extension().and_then(|p| p.to_str()) {
        Some("h") | Some("inl") => RING_INCLUDES.iter().any(cmp),
        Some("c") | Some("S") | Some("asm") => {
            RING_SRCS.iter().any(|(_, f)| cmp(f))
        },
        _ => true,
    };
    if !tracked {
        panic!("{:?} is not tracked in build.rs", p);
    }
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
