//! The `ken` command-line driver (scaffold).

fn main() {
    println!(
        "ken {} — verified topos-oriented language (scaffold)",
        env!("CARGO_PKG_VERSION")
    );
    println!("kernel {}", ken_kernel::version());
    println!("{}", ken_interp::describe());
    // Real subcommands (build, check, prove, repl) land with the elaborator,
    // prover, and interpreter work packages.
    let _ = ken_elaborator::kernel_version();
}
