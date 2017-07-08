extern crate lit;

const CRATE_PATH: &'static str = env!("CARGO_MANIFEST_DIR");

/// Information about a compiler.
#[derive(Debug)]
struct Compiler {
    executable_path: &'static str,
}

const AVR_GCC: Compiler = Compiler { executable_path: "avr-g++" };

fn main() {
    let compiler = AVR_GCC;

    lit::run::tests(|config| {
        config.add_search_path(format!("{}/tests", CRATE_PATH));
        config.add_extension("cpp");

        config.constants.insert("cxx".to_owned(), compiler.executable_path.to_owned());
    }).expect("failed tests");
}

