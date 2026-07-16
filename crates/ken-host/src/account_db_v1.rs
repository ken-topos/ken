//! Confined account-database boundary for effective-user home lookup.

#![allow(unsafe_code)]

use crate::{io_error_identity_v1, EffectiveUidSnapshotV1, HomeRootResolutionFailureV1};
use std::io;

pub(crate) trait AccountHomeLookupV1 {
    fn resolve_effective_user_home(
        &self,
        uid: EffectiveUidSnapshotV1,
    ) -> Result<Vec<u8>, HomeRootResolutionFailureV1>;
}

pub(crate) struct LibcAccountHomeLookupV1;

impl AccountHomeLookupV1 for LibcAccountHomeLookupV1 {
    fn resolve_effective_user_home(
        &self,
        uid: EffectiveUidSnapshotV1,
    ) -> Result<Vec<u8>, HomeRootResolutionFailureV1> {
        resolve_with_backend(uid, &LibcPasswdBackendV1)
    }
}

enum LookupAttemptV1 {
    Range,
    NoAccountRecord,
    BackendError(i32),
    InvalidRecord,
    Record { uid: u32, directory_offset: usize },
}

trait PasswdBackendV1 {
    fn lookup(&self, uid: u32, buffer: &mut [u8]) -> LookupAttemptV1;
}

struct LibcPasswdBackendV1;

impl PasswdBackendV1 for LibcPasswdBackendV1 {
    fn lookup(&self, uid: u32, buffer: &mut [u8]) -> LookupAttemptV1 {
        #[cfg(target_os = "linux")]
        {
            let mut record = std::mem::MaybeUninit::<libc::passwd>::zeroed();
            let mut result = std::ptr::null_mut();
            // SAFETY: `record`, `result`, and the initialized byte buffer are
            // valid for the call. The result pointer is compared with the
            // supplied record before any field is read.
            let status = unsafe {
                libc::getpwuid_r(
                    uid,
                    record.as_mut_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut result,
                )
            };
            if status == libc::ERANGE {
                return LookupAttemptV1::Range;
            }
            if status != 0 {
                return LookupAttemptV1::BackendError(status);
            }
            if result.is_null() {
                return LookupAttemptV1::NoAccountRecord;
            }
            if result != record.as_mut_ptr() {
                return LookupAttemptV1::InvalidRecord;
            }
            // SAFETY: success returned exactly the supplied initialized record.
            let record = unsafe { record.assume_init() };
            if record.pw_dir.is_null() {
                return LookupAttemptV1::InvalidRecord;
            }
            LookupAttemptV1::Record {
                uid: record.pw_uid,
                directory_offset: (record.pw_dir as usize).wrapping_sub(buffer.as_ptr() as usize),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (uid, buffer);
            LookupAttemptV1::BackendError(-1)
        }
    }
}

fn resolve_with_backend(
    uid: EffectiveUidSnapshotV1,
    backend: &impl PasswdBackendV1,
) -> Result<Vec<u8>, HomeRootResolutionFailureV1> {
    const INITIAL: usize = 1024;
    const LIMIT: usize = 1024 * 1024;
    let mut capacity = INITIAL;
    loop {
        let mut buffer = vec![0_u8; capacity];
        let (record_uid, offset) = match backend.lookup(uid.raw(), &mut buffer) {
            LookupAttemptV1::Range => {
                if capacity == LIMIT {
                    return Err(HomeRootResolutionFailureV1::AccountRecordTooLarge);
                }
                capacity = (capacity * 2).min(LIMIT);
                continue;
            }
            LookupAttemptV1::NoAccountRecord => {
                return Err(HomeRootResolutionFailureV1::NoAccountRecord);
            }
            LookupAttemptV1::BackendError(status) => {
                let error = io::Error::from_raw_os_error(status);
                return Err(HomeRootResolutionFailureV1::AccountLookup(
                    io_error_identity_v1(&error),
                ));
            }
            LookupAttemptV1::InvalidRecord => {
                return Err(HomeRootResolutionFailureV1::InvalidAccountRecord);
            }
            LookupAttemptV1::Record {
                uid,
                directory_offset,
            } => (uid, directory_offset),
        };
        if record_uid != uid.raw() || offset >= buffer.len() {
            return Err(HomeRootResolutionFailureV1::InvalidAccountRecord);
        }
        let tail = &buffer[offset..];
        let length = tail
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(HomeRootResolutionFailureV1::InvalidAccountRecord)?;
        let bytes = &tail[..length];
        if bytes.is_empty() || bytes[0] != b'/' {
            return Err(HomeRootResolutionFailureV1::InvalidAccountRecord);
        }
        return Ok(bytes.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    struct ScriptedBackend {
        attempts: RefCell<VecDeque<LookupAttemptV1>>,
        directory: Vec<u8>,
        calls: Cell<usize>,
    }

    impl PasswdBackendV1 for ScriptedBackend {
        fn lookup(&self, _uid: u32, buffer: &mut [u8]) -> LookupAttemptV1 {
            self.calls.set(self.calls.get() + 1);
            let attempt = self.attempts.borrow_mut().pop_front().unwrap();
            if let LookupAttemptV1::Record {
                directory_offset, ..
            } = attempt
            {
                let end = directory_offset.saturating_add(self.directory.len());
                if end <= buffer.len() {
                    buffer[directory_offset..end].copy_from_slice(&self.directory);
                }
            }
            attempt
        }
    }

    fn backend(attempts: Vec<LookupAttemptV1>, directory: &[u8]) -> ScriptedBackend {
        ScriptedBackend {
            attempts: RefCell::new(attempts.into()),
            directory: directory.to_vec(),
            calls: Cell::new(0),
        }
    }

    #[test]
    fn scripted_backend_covers_growth_cap_and_record_validation() {
        let uid = EffectiveUidSnapshotV1::scripted(1000);
        let success = backend(
            vec![
                LookupAttemptV1::Range,
                LookupAttemptV1::Record {
                    uid: 1000,
                    directory_offset: 7,
                },
            ],
            b"/accounts/test\0",
        );
        assert_eq!(
            resolve_with_backend(uid, &success).unwrap(),
            b"/accounts/test"
        );
        assert_eq!(success.calls.get(), 2);

        let capped = backend(
            std::iter::repeat_with(|| LookupAttemptV1::Range)
                .take(11)
                .collect(),
            b"",
        );
        assert_eq!(
            resolve_with_backend(uid, &capped),
            Err(HomeRootResolutionFailureV1::AccountRecordTooLarge)
        );
        let backend_error = io::Error::from_raw_os_error(5);
        for (attempt, expected) in [
            (
                LookupAttemptV1::NoAccountRecord,
                HomeRootResolutionFailureV1::NoAccountRecord,
            ),
            (
                LookupAttemptV1::BackendError(5),
                HomeRootResolutionFailureV1::AccountLookup(io_error_identity_v1(&backend_error)),
            ),
            (
                LookupAttemptV1::InvalidRecord,
                HomeRootResolutionFailureV1::InvalidAccountRecord,
            ),
            (
                LookupAttemptV1::Record {
                    uid: 1001,
                    directory_offset: 0,
                },
                HomeRootResolutionFailureV1::InvalidAccountRecord,
            ),
            (
                LookupAttemptV1::Record {
                    uid: 1000,
                    directory_offset: usize::MAX,
                },
                HomeRootResolutionFailureV1::InvalidAccountRecord,
            ),
        ] {
            assert_eq!(
                resolve_with_backend(uid, &backend(vec![attempt], b"/ok\0")),
                Err(expected)
            );
        }
        for directory in [b"\0".as_slice(), b"relative\0"] {
            assert_eq!(
                resolve_with_backend(
                    uid,
                    &backend(
                        vec![LookupAttemptV1::Record {
                            uid: 1000,
                            directory_offset: 0,
                        }],
                        directory,
                    ),
                ),
                Err(HomeRootResolutionFailureV1::InvalidAccountRecord)
            );
        }
        let unterminated = vec![b'/'; 1024];
        assert_eq!(
            resolve_with_backend(
                uid,
                &backend(
                    vec![LookupAttemptV1::Record {
                        uid: 1000,
                        directory_offset: 0,
                    }],
                    &unterminated,
                ),
            ),
            Err(HomeRootResolutionFailureV1::InvalidAccountRecord)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn production_lookup_is_owned_and_is_success_or_exact_no_record() {
        let uid = crate::observe_effective_uid_v1().expect("Linux euid snapshot");
        match resolve_with_backend(uid, &LibcPasswdBackendV1) {
            Ok(home) => assert!(home.starts_with(b"/") && !home.contains(&0)),
            Err(HomeRootResolutionFailureV1::NoAccountRecord) => {}
            Err(error) => panic!("unexpected account-database result: {error:?}"),
        }
    }
}
