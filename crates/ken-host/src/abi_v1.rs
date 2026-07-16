//! The sole raw native host ABI boundary.
//!
//! Every pointer is validated here and immediately converted into the typed
//! request vocabulary consumed by the ordinary safe dispatcher.  No pointer,
//! descriptor, or host identity enters Ken data.

#![allow(unsafe_code)]

use std::io::{self, Write};

use crate::{
    dispatch_host_op_v1, CanonicalOutcomeV1, CanonicalRequestV1, CapabilityGrantV1,
    CapabilityTableV1, ConsoleStreamV1, CreatePolicyV1, FileErrorCauseV1,
    HostEffectBackendV1, HostOpV1, IoErrorIdentityV1,
};

struct ProcessHost;

impl HostEffectBackendV1 for ProcessHost {
    fn console_write(
        &mut self,
        stream: ConsoleStreamV1,
        bytes: &[u8],
    ) -> Result<(), IoErrorIdentityV1> {
        let result = match stream {
            ConsoleStreamV1::Stdout => io::stdout().lock().write_all(bytes),
            ConsoleStreamV1::Stderr => io::stderr().lock().write_all(bytes),
            ConsoleStreamV1::Stdin => Err(io::ErrorKind::InvalidInput.into()),
        };
        result.map_err(map_io_error)
    }

    fn console_flush(&mut self, stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
        let result = match stream {
            ConsoleStreamV1::Stdout => io::stdout().lock().flush(),
            ConsoleStreamV1::Stderr => io::stderr().lock().flush(),
            ConsoleStreamV1::Stdin => Err(io::ErrorKind::InvalidInput.into()),
        };
        result.map_err(map_io_error)
    }

    fn console_is_terminal(&mut self, stream: ConsoleStreamV1) -> bool {
        use std::io::IsTerminal;
        match stream {
            ConsoleStreamV1::Stdin => io::stdin().is_terminal(),
            ConsoleStreamV1::Stdout => io::stdout().is_terminal(),
            ConsoleStreamV1::Stderr => io::stderr().is_terminal(),
        }
    }

    fn fs_read_file(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
    ) -> Result<Vec<u8>, FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }

    fn fs_write_file(
        &mut self,
        _grant: &CapabilityGrantV1,
        _path: &[u8],
        _create_policy: CreatePolicyV1,
        _bytes: &[u8],
    ) -> Result<(), FileErrorCauseV1> {
        Err(FileErrorCauseV1::Io(IoErrorIdentityV1::Unsupported))
    }
}

fn map_io_error(error: io::Error) -> IoErrorIdentityV1 {
    match error.kind() {
        io::ErrorKind::NotFound => IoErrorIdentityV1::NotFound,
        io::ErrorKind::PermissionDenied => IoErrorIdentityV1::PermissionDenied,
        io::ErrorKind::BrokenPipe => IoErrorIdentityV1::BrokenPipe,
        io::ErrorKind::Interrupted => IoErrorIdentityV1::Interrupted,
        io::ErrorKind::AlreadyExists => IoErrorIdentityV1::AlreadyExists,
        io::ErrorKind::InvalidInput => IoErrorIdentityV1::InvalidInput,
        io::ErrorKind::IsADirectory => IoErrorIdentityV1::IsDirectory,
        io::ErrorKind::NotADirectory => IoErrorIdentityV1::NotDirectory,
        io::ErrorKind::DirectoryNotEmpty => IoErrorIdentityV1::NotEmpty,
        io::ErrorKind::Unsupported => IoErrorIdentityV1::Unsupported,
        _ => IoErrorIdentityV1::Other(error.raw_os_error().unwrap_or(0)),
    }
}

fn stream(tag: u64) -> Option<ConsoleStreamV1> {
    match tag {
        0 => Some(ConsoleStreamV1::Stdin),
        1 => Some(ConsoleStreamV1::Stdout),
        2 => Some(ConsoleStreamV1::Stderr),
        _ => None,
    }
}

/// Private V1 dispatch symbol linked only into produced Ken executables.
///
/// `data` is borrowed for this call only.  Null is accepted exactly for an
/// empty slice; all other malformed fields fail before the shared dispatcher.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ken_host_dispatch_v1(
    _invocation: *const core::ffi::c_void,
    op: u64,
    data: *const u8,
    len: usize,
    auxiliary: u64,
) -> i64 {
    let Ok(op) = u16::try_from(op).ok().and_then(|op| HostOpV1::try_from(op).ok()).ok_or(()) else {
        return -3;
    };
    let Some(stream) = stream(auxiliary) else {
        return -1;
    };
    let bytes = if len == 0 {
        &[]
    } else {
        if data.is_null() {
            return -1;
        }
        // SAFETY: non-null was checked above; the generated caller owns this
        // immutable stack slice for the duration of this synchronous call.
        unsafe { std::slice::from_raw_parts(data, len) }
    };
    let request = match op {
        HostOpV1::ConsoleWrite => CanonicalRequestV1::ConsoleWrite {
            stream,
            bytes: bytes.to_vec(),
        },
        HostOpV1::ConsoleFlush => CanonicalRequestV1::ConsoleFlush { stream },
        HostOpV1::ConsoleIsTerminal => CanonicalRequestV1::ConsoleIsTerminal { stream },
        _ => return -3,
    };
    let mut host = ProcessHost;
    let result = dispatch_host_op_v1(
        &mut host,
        &CapabilityTableV1::default(),
        op,
        None,
        &request,
    );
    match result {
        Ok(reply) => match reply.outcome {
            CanonicalOutcomeV1::Success(crate::CanonicalReplyV1::Bool(value)) => i64::from(value),
            CanonicalOutcomeV1::Success(_) => 0,
            CanonicalOutcomeV1::Error(_) => -2,
        },
        Err(_) => -3,
    }
}
