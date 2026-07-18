//! Compiler-private checked scalar-join plan transport.

use crate::fnv1a_64;

pub const NATIVE_JOIN_PLAN_V1_HEADER: &[u8] = b"NativeJoinPlanV1\0";
pub const NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1: u64 = 0x4b4a_5245_5455_524e;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeJoinAnswerKindV1 {
    Int,
    Bool,
    StructuralNat,
    ExitCode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJoinPlanSiteV1 {
    pub site_id: u64,
    pub declaration: String,
    pub checked_result_type_fingerprint: u64,
    pub runtime_frame_fingerprint: u64,
    pub answer_kind: NativeJoinAnswerKindV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJoinPlanV1 {
    pub representation_rule_version: u32,
    pub sites: Vec<NativeJoinPlanSiteV1>,
}

impl NativeJoinPlanV1 {
    pub const REPRESENTATION_RULE_VERSION: u32 = 1;

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = NATIVE_JOIN_PLAN_V1_HEADER.to_vec();
        put_u32(&mut out, self.representation_rule_version);
        put_u64(&mut out, self.sites.len() as u64);
        for site in &self.sites {
            put_u64(&mut out, site.site_id);
            put_bytes(&mut out, site.declaration.as_bytes());
            put_u64(&mut out, site.checked_result_type_fingerprint);
            put_u64(&mut out, site.runtime_frame_fingerprint);
            out.push(match site.answer_kind {
                NativeJoinAnswerKindV1::Int => 0,
                NativeJoinAnswerKindV1::Bool => 1,
                NativeJoinAnswerKindV1::StructuralNat => 2,
                NativeJoinAnswerKindV1::ExitCode => 3,
            });
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        let Some(mut bytes) = bytes.strip_prefix(NATIVE_JOIN_PLAN_V1_HEADER) else {
            return Err("missing NativeJoinPlanV1 header");
        };
        let representation_rule_version = take_u32(&mut bytes)?;
        if representation_rule_version != Self::REPRESENTATION_RULE_VERSION {
            return Err("unsupported NativeJoinPlanV1 representation rule version");
        }
        let count = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| "NativeJoinPlanV1 site count overflows usize")?;
        let mut sites = Vec::with_capacity(count);
        for _ in 0..count {
            let site_id = take_u64(&mut bytes)?;
            let declaration = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "NativeJoinPlanV1 declaration is not UTF-8")?;
            let checked_result_type_fingerprint = take_u64(&mut bytes)?;
            let runtime_frame_fingerprint = take_u64(&mut bytes)?;
            let (tag, tail) = bytes
                .split_first()
                .ok_or("truncated NativeJoinPlanV1 answer kind")?;
            bytes = tail;
            let answer_kind = match tag {
                0 => NativeJoinAnswerKindV1::Int,
                1 => NativeJoinAnswerKindV1::Bool,
                2 => NativeJoinAnswerKindV1::StructuralNat,
                3 => NativeJoinAnswerKindV1::ExitCode,
                _ => return Err("unknown NativeJoinPlanV1 answer kind"),
            };
            sites.push(NativeJoinPlanSiteV1 {
                site_id,
                declaration,
                checked_result_type_fingerprint,
                runtime_frame_fingerprint,
                answer_kind,
            });
        }
        if !bytes.is_empty() {
            return Err("NativeJoinPlanV1 has trailing bytes");
        }
        let mut ids = sites.iter().map(|site| site.site_id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        if ids.len() != sites.len() {
            return Err("NativeJoinPlanV1 repeats a site identity");
        }
        Ok(Self {
            representation_rule_version,
            sites,
        })
    }

    pub fn transport_hash(&self) -> u64 {
        fnv1a_64(&self.canonical_bytes())
    }
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn take_u32(bytes: &mut &[u8]) -> Result<u32, &'static str> {
    let (head, tail) = bytes
        .split_at_checked(4)
        .ok_or("truncated NativeJoinPlanV1 u32")?;
    *bytes = tail;
    Ok(u32::from_le_bytes(head.try_into().expect("four bytes")))
}

fn take_u64(bytes: &mut &[u8]) -> Result<u64, &'static str> {
    let (head, tail) = bytes
        .split_at_checked(8)
        .ok_or("truncated NativeJoinPlanV1 u64")?;
    *bytes = tail;
    Ok(u64::from_le_bytes(head.try_into().expect("eight bytes")))
}

fn take_bytes<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let len = usize::try_from(take_u64(bytes)?)
        .map_err(|_| "NativeJoinPlanV1 byte length overflows usize")?;
    let (head, tail) = bytes
        .split_at_checked(len)
        .ok_or("truncated NativeJoinPlanV1 bytes")?;
    *bytes = tail;
    Ok(head)
}
