//! Scripted execution host over the real rooted POSIX interpreter host.
//!
//! Canonical observations are produced by `ken-interp` at its actual dispatch
//! seam. This wrapper supplies ambient behavior, descriptor operations, action
//! counters, and an independent scenario assertion script only.

use std::collections::VecDeque;
use std::io;
use std::path::Path;

use ken_elaborator::capabilities::{self, Cap};
use ken_host::{CreatePolicyV1, HostOpV1};
use ken_interp::{
    CapabilityDenied, ConsoleStream, FsOpKind, HostCreatePolicy, HostDirEntry, HostFileMetadata,
    HostHandler, HostRead, PosixHost, Resolution, ResolveError,
};
use num_bigint::BigInt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AmbientScript {
    pub stdin: Vec<u8>,
    pub stdin_is_terminal: bool,
    pub stdout_is_terminal: bool,
    pub stderr_is_terminal: bool,
    pub wall_clock_nanoseconds: Vec<BigInt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpectedFsEffect {
    ReadFile {
        path: Vec<u8>,
    },
    WriteFile {
        path: Vec<u8>,
        create_policy: CreatePolicyV1,
        bytes: Vec<u8>,
    },
}

impl ExpectedFsEffect {
    fn operation(&self) -> HostOpV1 {
        match self {
            Self::ReadFile { .. } => HostOpV1::FsReadFile,
            Self::WriteFile { .. } => HostOpV1::FsWriteFile,
        }
    }

    fn path(&self) -> &[u8] {
        match self {
            Self::ReadFile { path } | Self::WriteFile { path, .. } => path,
        }
    }

    fn resolve_kind(&self) -> FsOpKind {
        match self {
            Self::ReadFile { .. } => FsOpKind::Read,
            Self::WriteFile { .. } => FsOpKind::Write,
        }
    }
}

pub struct ScriptedPosixHost {
    inner: PosixHost,
    scoped_cap: Option<(capabilities::Authority, Cap)>,
    stdin: VecDeque<u8>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    terminals: [bool; 3],
    wall_clock_nanoseconds: VecDeque<BigInt>,
    denials: Vec<CapabilityDenied>,
    fs_actions_after_resolve: u64,
    expected_fs: VecDeque<ExpectedFsEffect>,
    pending_fs: Option<ExpectedFsEffect>,
    assertion_error: Option<String>,
}

impl ScriptedPosixHost {
    pub fn new_at(root: impl AsRef<Path>, script: AmbientScript) -> Self {
        Self {
            inner: PosixHost::new_at(root),
            scoped_cap: None,
            stdin: script.stdin.into(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            terminals: [
                script.stdin_is_terminal,
                script.stdout_is_terminal,
                script.stderr_is_terminal,
            ],
            wall_clock_nanoseconds: script.wall_clock_nanoseconds.into(),
            denials: Vec::new(),
            fs_actions_after_resolve: 0,
            expected_fs: VecDeque::new(),
            pending_fs: None,
            assertion_error: None,
        }
    }

    pub fn new_scoped(
        root: impl AsRef<Path>,
        script: AmbientScript,
        authority: capabilities::Authority,
        relative_root: &[u8],
        rights: capabilities::RightSet,
        symlink: capabilities::SymlinkPolicy,
        expected_fs: Vec<ExpectedFsEffect>,
    ) -> io::Result<Self> {
        let mut host = Self::new_at(root, script);
        let cap = host
            .inner
            .mint_scoped_fs_cap(authority, relative_root, rights, symlink)?;
        host.scoped_cap = Some((authority, cap));
        host.expected_fs = expected_fs.into();
        Ok(host)
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn denials(&self) -> &[CapabilityDenied] {
        &self.denials
    }

    pub fn fs_actions_after_resolve(&self) -> u64 {
        self.fs_actions_after_resolve
    }

    /// Completes the independent execution script assertion. No value from
    /// this script enters `EffectObservationV1`.
    pub fn finish_assertions(&self) -> Result<(), String> {
        if let Some(error) = &self.assertion_error {
            return Err(error.clone());
        }
        if self.pending_fs.is_some() || !self.expected_fs.is_empty() {
            return Err(format!(
                "interpreter did not consume the scenario FS assertion script: pending={:?}, remaining={:?}",
                self.pending_fs, self.expected_fs
            ));
        }
        Ok(())
    }

    fn fail_assertion(&mut self, message: impl Into<String>) {
        if self.assertion_error.is_none() {
            self.assertion_error = Some(message.into());
        }
    }

    fn take_pending(&mut self, operation: HostOpV1) -> Option<ExpectedFsEffect> {
        let expected = self.pending_fs.take();
        if expected.as_ref().map(ExpectedFsEffect::operation) != Some(operation) {
            self.fail_assertion(format!(
                "unexpected interpreter FS leaf {operation:?}; pending={expected:?}"
            ));
            None
        } else {
            expected
        }
    }
}

impl HostHandler for ScriptedPosixHost {
    type Handle = <PosixHost as HostHandler>::Handle;

    fn mint_fs_cap(&self, authority: capabilities::Authority) -> Cap {
        match &self.scoped_cap {
            Some((expected, cap)) if *expected == authority => cap.clone(),
            _ => self.inner.mint_fs_cap(authority),
        }
    }

    fn console_read(&mut self, stream: ConsoleStream, limit: usize) -> io::Result<HostRead> {
        if stream != ConsoleStream::Stdin {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stream is not readable",
            ));
        }
        if limit == 0 {
            return Ok(HostRead::Chunk(Vec::new()));
        }
        if self.stdin.is_empty() {
            return Ok(HostRead::Eof);
        }
        let count = limit.min(self.stdin.len());
        Ok(HostRead::Chunk(self.stdin.drain(..count).collect()))
    }

    fn console_write(&mut self, stream: ConsoleStream, bytes: &[u8]) -> io::Result<()> {
        match stream {
            ConsoleStream::Stdout => self.stdout.extend_from_slice(bytes),
            ConsoleStream::Stderr => self.stderr.extend_from_slice(bytes),
            ConsoleStream::Stdin => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "stream is not writable",
                ));
            }
        }
        Ok(())
    }

    fn console_flush(&mut self, _stream: ConsoleStream) -> io::Result<()> {
        Ok(())
    }

    fn console_is_terminal(&mut self, stream: ConsoleStream) -> bool {
        self.terminals[stream_index(stream)]
    }

    fn clock_wall_now(&mut self) -> BigInt {
        self.wall_clock_nanoseconds
            .pop_front()
            .expect("PX6 Clock.WallNow requires an explicit scripted response")
    }

    fn fs_denied(&mut self, denial: CapabilityDenied) {
        self.denials.push(denial.clone());
        if self.expected_fs.pop_front().is_none() {
            self.fail_assertion(format!("unexpected FS denial: {denial:?}"));
        }
    }

    fn fs_after_resolve(&mut self) {
        self.fs_actions_after_resolve += 1;
    }

    fn fs_resolve(
        &mut self,
        root: &capabilities::FsHandle,
        components: &[Vec<u8>],
        op: FsOpKind,
        symlink: capabilities::SymlinkPolicy,
    ) -> Result<Resolution<Self::Handle>, ResolveError> {
        let expected = self.expected_fs.front().cloned();
        if let Some(expected) = &expected {
            let expected_components = expected
                .path()
                .split(|byte| *byte == b'/')
                .filter(|part| !part.is_empty() && *part != b".")
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if expected.resolve_kind() != op || expected_components != components {
                self.fail_assertion(format!(
                    "interpreter FS resolution diverged: expected={expected:?}, op={op:?}, components={components:?}"
                ));
            }
        } else {
            self.fail_assertion(format!(
                "interpreter performed unplanned FS resolution: op={op:?}, components={components:?}"
            ));
        }
        let result = self.inner.fs_resolve(root, components, op, symlink);
        match &result {
            Ok(_) => self.pending_fs = self.expected_fs.pop_front(),
            Err(ResolveError::Io(_)) => {
                self.expected_fs.pop_front();
            }
            Err(ResolveError::Denied(_)) => {}
        }
        result
    }

    fn fs_read_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<u8>> {
        self.take_pending(HostOpV1::FsReadFile);
        self.inner.fs_read_at(handle)
    }

    fn fs_write_at(
        &mut self,
        handle: &Self::Handle,
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        let expected = self.take_pending(HostOpV1::FsWriteFile);
        verify_write_assertion(self, expected.as_ref(), policy, bytes);
        self.inner.fs_write_at(handle, policy, bytes)
    }

    fn fs_create_file_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        let expected = self.take_pending(HostOpV1::FsWriteFile);
        verify_write_assertion(self, expected.as_ref(), policy, bytes);
        self.inner.fs_create_file_at(parent, leaf, policy, bytes)
    }

    fn fs_append_at(&mut self, handle: &Self::Handle, bytes: &[u8]) -> io::Result<()> {
        self.inner.fs_append_at(handle, bytes)
    }

    fn fs_create_append_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        bytes: &[u8],
    ) -> io::Result<()> {
        self.inner.fs_create_append_at(parent, leaf, bytes)
    }

    fn fs_metadata_at(&mut self, handle: &Self::Handle) -> io::Result<HostFileMetadata> {
        self.inner.fs_metadata_at(handle)
    }

    fn fs_read_directory_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<HostDirEntry>> {
        self.inner.fs_read_directory_at(handle)
    }

    fn fs_create_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()> {
        self.inner.fs_create_directory_at(parent, leaf, recursive)
    }

    fn fs_remove_file_at(&mut self, parent: &Self::Handle, leaf: &[u8]) -> io::Result<()> {
        self.inner.fs_remove_file_at(parent, leaf)
    }

    fn fs_remove_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()> {
        self.inner.fs_remove_directory_at(parent, leaf, recursive)
    }

    fn fs_rename_at(
        &mut self,
        from_parent: &Self::Handle,
        from_leaf: &[u8],
        to_parent: &Self::Handle,
        to_leaf: &[u8],
    ) -> io::Result<()> {
        self.inner
            .fs_rename_at(from_parent, from_leaf, to_parent, to_leaf)
    }
}

fn verify_write_assertion(
    host: &mut ScriptedPosixHost,
    expected: Option<&ExpectedFsEffect>,
    policy: HostCreatePolicy,
    bytes: &[u8],
) {
    let expected_policy = match policy {
        HostCreatePolicy::CreateNew => CreatePolicyV1::CreateNew,
        HostCreatePolicy::CreateOrTruncate => CreatePolicyV1::CreateOrTruncate,
        HostCreatePolicy::CreateOrKeep => CreatePolicyV1::CreateOrKeep,
    };
    if !matches!(
        expected,
        Some(ExpectedFsEffect::WriteFile { create_policy, bytes: expected_bytes, .. })
            if *create_policy == expected_policy && expected_bytes == bytes
    ) {
        host.fail_assertion(format!(
            "interpreter FS write payload diverged: expected={expected:?}, policy={policy:?}, bytes={bytes:?}"
        ));
    }
}

fn stream_index(stream: ConsoleStream) -> usize {
    match stream {
        ConsoleStream::Stdin => 0,
        ConsoleStream::Stdout => 1,
        ConsoleStream::Stderr => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TwinRealRoots;

    #[test]
    fn scripted_console_is_execution_only() {
        let roots = TwinRealRoots::create(&[]).expect("roots");
        let mut host = ScriptedPosixHost::new_at(
            roots.interpreter(),
            AmbientScript {
                stdin: vec![0xff, b'a', 0],
                ..AmbientScript::default()
            },
        );
        assert_eq!(
            host.console_read(ConsoleStream::Stdin, 2).expect("read"),
            HostRead::Chunk(vec![0xff, b'a'])
        );
        host.console_write(ConsoleStream::Stdout, &[0, 0xfe])
            .expect("stdout");
        assert_eq!(host.stdout(), &[0, 0xfe]);
        host.finish_assertions().expect("empty assertion script");
    }

    #[test]
    #[should_panic(expected = "explicit scripted response")]
    fn unscripted_clock_read_fails_loudly() {
        let roots = TwinRealRoots::create(&[]).expect("roots");
        let mut host = ScriptedPosixHost::new_at(roots.interpreter(), AmbientScript::default());
        let _ = host.clock_wall_now();
    }
}
