extern crate lit;
extern crate clap;

use clap::{App, Arg};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

const CRATE_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

/// Information about a compiler.
#[derive(Debug)]
struct Compiler {
    /// The C compiler.
    cc: PathBuf,
    /// The C++ compiler.
    cxx: PathBuf,
    /// Flags to be passed to both C and C++ compilers.
    compiler_flags: Vec<&'static str>,
}

fn main() {
    let matches = App::new("avr-lit")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("avr-gcc")
            .long("avr-gcc")
            .help("Compile with the system avr-gcc"))
        .arg(Arg::with_name("llvm-sysroot")
            .long("llvm-sysroot")
            .value_name("SYSROOT")
            .help("Compile tests with an LLVM sysroot")
            .takes_value(true))
        .arg(Arg::with_name("TESTS")
            .help("Sets the tests to run")
            .required(false)
            .index(1))
        .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))
        .get_matches();

    let avr_gcc_enabled = matches.is_present("avr-gcc");
    let llvm_sysroot = matches.value_of("llvm-sysroot");

    let compiler = match (avr_gcc_enabled, llvm_sysroot) {
        // GCC enabled, no LLVM sysroot given
        (true, None) => match self::gnu_tools() {
            Some(compiler) => compiler,
            None => {
                eprintln!("error: cannot find avr-gcc in PATH");
                process::exit(1);
            },
        },
        // GCC not enabled, LLVM sysroot given
        (false, Some(llvm_sysroot)) => {
            match self::detect_compiler(Path::new(llvm_sysroot)) {
                Some(compiler) => compiler,
                None => {
                    eprintln!("error: LLVM sysroot does not look like an LLVM sysroot");
                    process::exit(1);
                },
            }
        },
        (true, Some(..)) => {
            eprintln!("error: cannot compile with both GCC and LLVM");
            process::exit(1);
        },
        (false, None) => {
            eprintln!("error: either LLVM or GCC must be chosen");
            process::exit(1);
        },
    };

    // Gets a value for config if supplied by user, or defaults to "default.conf"

    lit::run::tests(|config| {
        if let Some(tests_path) = matches.value_of("TESTS") {
            config.add_search_path(tests_path);
        } else {
            // No tests explicitly passed, default to all.
            config.add_search_path(format!("{}/tests", CRATE_PATH));
        }
        config.add_extension("c");
        config.add_extension("cpp");

        insert_constants(&mut config.constants, &compiler);
    }).expect("failed tests");
}

fn insert_constants(constants: &mut HashMap<String, String>, compiler: &Compiler) {
    constants.insert("cc".to_owned(),
                     compiler.cc.display().to_string());
    constants.insert("cflags".to_owned(), compiler.compiler_flags.join(" "));

    constants.insert("cxx".to_owned(),
                     compiler.cxx.display().to_string());
    constants.insert("cxxflags".to_owned(), compiler.compiler_flags.join(" "));
}

fn gnu_tools() -> Option<Compiler> {
    match util::find_in_path("avr-gcc") {
        Some(cc_path) => {
            let exec_dir = cc_path.parent().unwrap();

            Some(Compiler {
                cc: cc_path.to_owned(),
                cxx: exec_dir.join("avr-g++"),
                compiler_flags: all_compiler_flags(&[]),
            })
        },
        None => None,
    }
}

fn all_compiler_flags(other_flags: &[&'static str]) -> Vec<&'static str> {
    let mut flags = vec![
        "-mmcu=atmega328p",
        "-Isrc/libavrlit/stdlib",
        "-std=c++11",
        "-ffunction-sections",
        "-g",
    ];

    flags.extend(other_flags);
    flags
}

fn detect_compiler(sysroot: &Path) -> Option<Compiler> {
    let bin_dir = sysroot.join("bin");

    if bin_dir.join("clang").exists() {
        Some(Compiler {
            cc: bin_dir.join("clang"),
            cxx: bin_dir.join("clang++"),
            compiler_flags: all_compiler_flags(&[
                "-target", "avr-unknown-unknown",
                "--verbose", "-O",
            ]),
        })
    } else {
        None
    }
}

mod util {
    use std::path::{Path, PathBuf};
    use std::env;

    // Adapted from
    // https://stackoverflow.com/a/37499032
    pub fn find_in_path<P>(exe_name: P) -> Option<PathBuf>
        where P: AsRef<Path> {
        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths).filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            }).next()
        })
    }
}

