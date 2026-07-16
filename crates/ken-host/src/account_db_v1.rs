//! Confined account-database boundary for effective-user home lookup.

#![allow(unsafe_code)]

use crate::{EffectiveUidSnapshotV1, HomeRootResolutionFailureV1};

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
        resolve_effective_user_home(uid)
    }
}

/// Resolve the executing effective UID through the system account database.
///
/// The returned bytes are owned. No libc pointer, borrow, or type crosses this
/// module boundary.
fn resolve_effective_user_home(
    uid: EffectiveUidSnapshotV1,
) -> Result<Vec<u8>, HomeRootResolutionFailureV1> {
    #[cfg(target_os = "linux")]
    {
        const INITIAL: usize = 1024;
        const LIMIT: usize = 1024 * 1024;
        let mut capacity = INITIAL;
        loop {
            let mut buffer = vec![0_u8; capacity];
            let mut record = std::mem::MaybeUninit::<libc::passwd>::zeroed();
            let mut result = std::ptr::null_mut();
            // SAFETY: `record`, `result`, and the initialized byte buffer are
            // valid for the call. All returned pointers are validated and
            // copied before either local allocation is dropped.
            let status = unsafe {
                libc::getpwuid_r(
                    uid.raw(),
                    record.as_mut_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut result,
                )
            };
            if status == libc::ERANGE {
                if capacity == LIMIT {
                    return Err(HomeRootResolutionFailureV1::BufferCapacityExceeded);
                }
                capacity = (capacity * 2).min(LIMIT);
                continue;
            }
            if status != 0 {
                return Err(HomeRootResolutionFailureV1::NssError(status));
            }
            if result.is_null() {
                return Err(HomeRootResolutionFailureV1::NoEntry);
            }
            if result != record.as_mut_ptr() {
                return Err(HomeRootResolutionFailureV1::InvalidHomeDirectory);
            }
            // SAFETY: a successful call returned exactly the supplied record.
            let record = unsafe { record.assume_init() };
            if record.pw_uid != uid.raw() || record.pw_dir.is_null() {
                return Err(HomeRootResolutionFailureV1::InvalidHomeDirectory);
            }
            let start = buffer.as_ptr() as usize;
            let end = start
                .checked_add(buffer.len())
                .ok_or(HomeRootResolutionFailureV1::InvalidHomeDirectory)?;
            let directory = record.pw_dir as usize;
            if directory < start || directory >= end {
                return Err(HomeRootResolutionFailureV1::InvalidHomeDirectory);
            }
            let offset = directory - start;
            let tail = &buffer[offset..];
            let length = tail
                .iter()
                .position(|byte| *byte == 0)
                .ok_or(HomeRootResolutionFailureV1::InvalidHomeDirectory)?;
            let bytes = &tail[..length];
            if bytes.is_empty() || bytes[0] != b'/' {
                return Err(HomeRootResolutionFailureV1::InvalidHomeDirectory);
            }
            return Ok(bytes.to_vec());
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = uid;
        Err(HomeRootResolutionFailureV1::NssError(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn production_lookup_is_owned_and_is_success_or_exact_no_entry() {
        let uid = crate::observe_effective_uid_v1().expect("Linux euid snapshot");
        match resolve_effective_user_home(uid) {
            Ok(home) => assert!(home.starts_with(b"/") && !home.contains(&0)),
            Err(HomeRootResolutionFailureV1::NoEntry) => {}
            Err(error) => panic!("unexpected account-database result: {error:?}"),
        }
    }
}
