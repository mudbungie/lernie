//! **Re-emit the checked-in vector source** from the generator that defines it.
//!
//! `make icon` is the door; this is the machinery. It exists as an example
//! rather than as a verb because it is a build act and not a thing an operator
//! does to a seat — and rather than as a test because a test that writes into
//! the tree is a test that cannot fail.
//!
//! The pinning half is `lernie::mark`'s own suite, which asserts the file this
//! writes still equals what the generator produces. So the artifact is a
//! DERIVATION: edit `src/mark.rs`, run this, and commit both.

fn main() -> std::process::ExitCode {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: cargo run --example icon -- <dir>");
        return std::process::ExitCode::FAILURE;
    };
    let at = std::path::Path::new(&dir).join(format!("{}.svg", lernie::mark::APP_ID));
    match std::fs::write(&at, lernie::mark::svg()) {
        Ok(()) => {
            println!("{}", at.display());
            std::process::ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{}: {why}", at.display());
            std::process::ExitCode::FAILURE
        }
    }
}
