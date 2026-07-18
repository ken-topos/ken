//! Private PX5 native-effect invocation substrate.
//!
//! The checked plan is contained by the checked-core identity; it deliberately
//! contains no enclosing digest. Hashes for the plan, core, and artifact are
//! parent-owned adjacent bindings.

#![allow(dead_code)]

use ken_host::{
    assert_host_effect_abi_identity, assert_target_abi_identity, dispatch_host_op_v1,
    CanonicalRequestV1, CapabilityTableV1, CapabilityTokenV1, EffectObservation, FsDeltaV1,
    HostDispatchReplyV1, HostEffectBackendV1, HostOpV1, ResourceInputsV1, ResourceTableV1,
    TerminalErrorV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeEntrypointBindingsV1 {
    pub native_entrypoint_plan_hash: u64,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub target_abi_hash: [u8; 32],
    pub host_effect_abi_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeEffectCallV1 {
    pub operation: HostOpV1,
    pub capability: Option<(String, CapabilityTokenV1)>,
    pub resources: ResourceInputsV1,
    pub request: CanonicalRequestV1,
}

#[derive(Clone, Debug, Default)]
pub struct ResponseArenaV1 {
    replies: Vec<HostDispatchReplyV1>,
}

impl ResponseArenaV1 {
    pub fn append(&mut self, reply: HostDispatchReplyV1) -> usize {
        let index = self.replies.len();
        self.replies.push(reply);
        index
    }

    pub fn get(&self, index: usize) -> Option<&HostDispatchReplyV1> {
        self.replies.get(index)
    }
}

/// Host-owned, call-scoped state. It is never represented as ordinary Ken
/// data, and its capability table cannot be read by generated code.
pub struct KenNativeInvocationV1<B> {
    pub bindings: NativeEntrypointBindingsV1,
    pub backend: B,
    pub capabilities: CapabilityTableV1,
    pub resources: ResourceTableV1,
    pub response_arena: ResponseArenaV1,
    pub observation: EffectObservation,
}

impl<B: HostEffectBackendV1> KenNativeInvocationV1<B> {
    pub fn initialize(
        bindings: NativeEntrypointBindingsV1,
        backend: B,
        capabilities: CapabilityTableV1,
    ) -> Result<Self, TerminalErrorV1> {
        assert_target_abi_identity(bindings.target_abi_hash)
            .map_err(|_| TerminalErrorV1::TargetAbiMismatch)?;
        assert_host_effect_abi_identity(bindings.host_effect_abi_hash)?;
        if bindings.native_entrypoint_plan_hash == 0
            || bindings.core_semantic_hash == 0
            || bindings.artifact_hash == 0
        {
            return Err(TerminalErrorV1::MalformedHostAbiField);
        }
        Ok(Self {
            bindings,
            backend,
            capabilities,
            resources: ResourceTableV1::default(),
            response_arena: ResponseArenaV1::default(),
            observation: EffectObservation {
                stdout: Vec::new(),
                stderr: Vec::new(),
                filesystem_delta: Vec::<FsDeltaV1>::new(),
                terminal_error: None,
                effect_trace: Vec::new(),
                terminal_exit: ken_host::TerminalExitClass::ReturnedError,
                exit_status: 1,
            },
        })
    }

    pub fn dispatch(&mut self, call: NativeEffectCallV1) -> Result<usize, TerminalErrorV1> {
        let token = match call.capability {
            Some((_, token)) => Some(token),
            None => None,
        };
        let reply = dispatch_host_op_v1(
            &mut self.backend,
            &self.capabilities,
            &mut self.resources,
            call.operation,
            token,
            call.resources,
            &call.request,
        )?;
        self.observation
            .effect_trace
            .push(ken_host::effect_event_from_dispatch(
                self.observation.effect_trace.len() as u64,
                call.operation,
                call.request,
                &reply,
            ));
        Ok(self.response_arena.append(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ken_host::{
        CanonicalOutcomeV1, CanonicalReplyV1, CapabilityGrantV1, CapabilityTraceIdentity,
        CreatePolicyV1, FileErrorCauseV1, IoErrorIdentityV1, HOST_EFFECT_ABI_V1_HASH,
        RIGHT_READ_V1, RIGHT_WRITE_V1, TARGET_ABI_MANIFEST_HASH,
    };

    #[derive(Default)]
    struct MockHost {
        stdout: Vec<u8>,
        file: Vec<u8>,
        calls: usize,
    }

    impl HostEffectBackendV1 for MockHost {
        fn console_write(
            &mut self,
            _stream: ken_host::ConsoleStreamV1,
            bytes: &[u8],
        ) -> Result<(), IoErrorIdentityV1> {
            self.calls += 1;
            self.stdout.extend_from_slice(bytes);
            Ok(())
        }

        fn console_flush(
            &mut self,
            _stream: ken_host::ConsoleStreamV1,
        ) -> Result<(), IoErrorIdentityV1> {
            self.calls += 1;
            Ok(())
        }

        fn console_is_terminal(&mut self, _stream: ken_host::ConsoleStreamV1) -> bool {
            self.calls += 1;
            false
        }

        fn fs_read_file(
            &mut self,
            _grant: &CapabilityGrantV1,
            _path: &[u8],
        ) -> Result<Vec<u8>, FileErrorCauseV1> {
            self.calls += 1;
            Ok(self.file.clone())
        }

        fn fs_write_file(
            &mut self,
            _grant: &CapabilityGrantV1,
            _path: &[u8],
            _create_policy: CreatePolicyV1,
            bytes: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.calls += 1;
            self.file = bytes.to_vec();
            Ok(())
        }
    }

    fn invocation(rights: u8) -> (KenNativeInvocationV1<MockHost>, CapabilityTokenV1) {
        let bindings = NativeEntrypointBindingsV1 {
            native_entrypoint_plan_hash: 3,
            core_semantic_hash: 1,
            artifact_hash: 2,
            target_abi_hash: TARGET_ABI_MANIFEST_HASH,
            host_effect_abi_hash: HOST_EFFECT_ABI_V1_HASH,
        };
        let mut capabilities = CapabilityTableV1::default();
        let token = capabilities.insert(CapabilityGrantV1 {
            identity: CapabilityTraceIdentity("fsCap".to_string()),
            capability: ken_host::Cap::mint_scoped(
                ken_host::AUTH_FULL,
                "FS",
                ken_host::FsScope::root(
                    ken_host::RightSet::from_bits(rights),
                    ken_host::FsHandle::Virtual(0),
                    ken_host::FsIdentity::Virtual(0),
                    ken_host::SymlinkPolicy::NoFollow,
                ),
            ),
        });
        (
            KenNativeInvocationV1::initialize(bindings, MockHost::default(), capabilities).unwrap(),
            token,
        )
    }

    #[test]
    fn checked_plan_identity_fails_before_host_construction() {
        let bindings = NativeEntrypointBindingsV1 {
            native_entrypoint_plan_hash: 0,
            core_semantic_hash: 1,
            artifact_hash: 2,
            target_abi_hash: TARGET_ABI_MANIFEST_HASH,
            host_effect_abi_hash: HOST_EFFECT_ABI_V1_HASH,
        };
        let result = KenNativeInvocationV1::initialize(
            bindings,
            MockHost::default(),
            CapabilityTableV1::default(),
        );
        assert!(matches!(
            result,
            Err(TerminalErrorV1::MalformedHostAbiField)
        ));
    }

    #[test]
    fn five_lane_dispatch_is_ordered_and_response_arena_is_invocation_lived() {
        let (mut invocation, token) = invocation(RIGHT_READ_V1 | RIGHT_WRITE_V1);
        let write = invocation
            .dispatch(NativeEffectCallV1 {
                operation: HostOpV1::FsWriteFile,
                capability: Some(("fsCap".to_string(), token)),
                resources: ResourceInputsV1::None,
                request: CanonicalRequestV1::FsWriteFile {
                    path: b"a".to_vec(),
                    create_policy: CreatePolicyV1::CreateOrTruncate,
                    bytes: b"payload".to_vec(),
                },
            })
            .unwrap();
        let read = invocation
            .dispatch(NativeEffectCallV1 {
                operation: HostOpV1::FsReadFile,
                capability: Some(("fsCap".to_string(), token)),
                resources: ResourceInputsV1::None,
                request: CanonicalRequestV1::FsReadFile {
                    path: b"a".to_vec(),
                },
            })
            .unwrap();
        assert_eq!(invocation.observation.effect_trace[0].sequence, 0);
        assert_eq!(invocation.observation.effect_trace[1].sequence, 1);
        assert!(matches!(
            invocation.response_arena.get(write).unwrap().outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::Unit)
        ));
        assert_eq!(
            invocation.response_arena.get(read).unwrap().outcome,
            CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(b"payload".to_vec()))
        );
    }

    #[test]
    fn denial_precedes_host_action() {
        let (mut invocation, token) = invocation(RIGHT_READ_V1);
        invocation
            .dispatch(NativeEffectCallV1 {
                operation: HostOpV1::FsWriteFile,
                capability: Some(("fsCap".to_string(), token)),
                resources: ResourceInputsV1::None,
                request: CanonicalRequestV1::FsWriteFile {
                    path: b"a".to_vec(),
                    create_policy: CreatePolicyV1::CreateNew,
                    bytes: b"no".to_vec(),
                },
            })
            .unwrap();
        assert_eq!(invocation.backend.calls, 0);
        assert!(matches!(
            invocation.observation.effect_trace[0].outcome,
            CanonicalOutcomeV1::Error(_)
        ));
    }
}
