//! # ken-elaborator — surface → kernel
//!
//! Parses Ken surface syntax and elaborates it into `ken-kernel` terms, then
//! (work packages **V1–V4**) generates proof obligations from surface
//! specifications and drives the prover backend. The elaborator may eventually
//! self-host; the kernel it targets does not.
//!
//! Status: scaffold. Minimal elaborator lands in **V0**.

/// Placeholder: the kernel version this elaborator targets.
pub fn kernel_version() -> &'static str {
    ken_kernel::version()
}
