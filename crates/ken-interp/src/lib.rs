//! # ken-interp — reference interpreter
//!
//! The initial execution backend: a tree-walking/bytecode interpreter that
//! defines Ken's **reference semantics**. Native codegen comes later and is
//! differential-tested against this interpreter, which remains the oracle.
//! Also home to the content-addressed runtime value model.
//!
//! Status: scaffold. Interpreter lands in **X1**; runtime hardening in **X2**.

/// Placeholder entry point for the interpreter.
pub fn describe() -> &'static str {
    "ken reference interpreter (scaffold)"
}
