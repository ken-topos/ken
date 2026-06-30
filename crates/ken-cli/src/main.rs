//! The `ken` command-line driver.

mod repl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "repl" => repl::run(),
        "version" | "--version" | "-V" => {
            println!("ken {} — verified topos-oriented language", env!("CARGO_PKG_VERSION"));
            println!("kernel {}", ken_kernel::version());
            println!("{}", ken_interp::describe());
        }
        "" | "--help" | "-h" | "help" => print_help(),
        unknown => {
            eprintln!("ken: unknown subcommand '{}' — try 'ken help'", unknown);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("ken {} — verified topos-oriented language", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: ken <subcommand>");
    println!();
    println!("Subcommands:");
    println!("  repl      Start the interactive REPL (the Little Prover loop)");
    println!("  version   Print version and kernel information");
    println!("  help      Print this message");
}
