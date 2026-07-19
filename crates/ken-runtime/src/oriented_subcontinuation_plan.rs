//! Compiler-private checked oriented-subcontinuation plan transport.

use std::collections::{BTreeMap, BTreeSet};

use crate::fnv1a_64;

pub const ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER: &[u8] =
    b"OrientedSubcontinuationPlanV1\0";
pub const CHECKED_ANSWER_INTERFACE_V1_HEADER: &[u8] = b"CheckedAnswerInterfaceV1\0";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckedAnswerInterfaceV1 {
    /// Canonical checked bytes are the authority. A fingerprint alone is not.
    pub canonical: Vec<u8>,
}

impl CheckedAnswerInterfaceV1 {
    pub fn new(canonical: Vec<u8>) -> Result<Self, &'static str> {
        if !canonical.starts_with(CHECKED_ANSWER_INTERFACE_V1_HEADER) {
            return Err("checked answer interface has no canonical header");
        }
        Ok(Self { canonical })
    }

    pub fn fingerprint(&self) -> u64 {
        fnv1a_64(&self.canonical)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrientedControlWitnessV1 {
    DistinguishedRoot,
    ParentFrame(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientedSubcontinuationFramePlanV1 {
    pub frame_id: u64,
    pub segment_site_id: u64,
    pub declaration: String,
    pub checked_occurrence_path: Vec<u64>,
    pub semantic_position: u64,
    pub input_interface: CheckedAnswerInterfaceV1,
    pub output_interface: CheckedAnswerInterfaceV1,
    pub runtime_frame_fingerprint: u64,
    pub occurrence_binding_fingerprint: u64,
    pub control_witness: OrientedControlWitnessV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientedSubcontinuationPlanV1 {
    pub representation_rule_version: u32,
    pub frames: Vec<OrientedSubcontinuationFramePlanV1>,
}

impl OrientedSubcontinuationPlanV1 {
    pub const REPRESENTATION_RULE_VERSION: u32 = 1;

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER.to_vec();
        put_u32(&mut out, self.representation_rule_version);
        put_u64(&mut out, self.frames.len() as u64);
        for frame in &self.frames {
            put_u64(&mut out, frame.frame_id);
            put_u64(&mut out, frame.segment_site_id);
            put_bytes(&mut out, frame.declaration.as_bytes());
            put_u64(&mut out, frame.checked_occurrence_path.len() as u64);
            for value in &frame.checked_occurrence_path {
                put_u64(&mut out, *value);
            }
            put_u64(&mut out, frame.semantic_position);
            put_bytes(&mut out, &frame.input_interface.canonical);
            put_bytes(&mut out, &frame.output_interface.canonical);
            put_u64(&mut out, frame.runtime_frame_fingerprint);
            put_u64(&mut out, frame.occurrence_binding_fingerprint);
            match frame.control_witness {
                OrientedControlWitnessV1::DistinguishedRoot => out.push(0),
                OrientedControlWitnessV1::ParentFrame(parent) => {
                    out.push(1);
                    put_u64(&mut out, parent);
                }
            }
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        let Some(mut bytes) = bytes.strip_prefix(ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER)
        else {
            return Err("missing OrientedSubcontinuationPlanV1 header");
        };
        let representation_rule_version = take_u32(&mut bytes)?;
        if representation_rule_version != Self::REPRESENTATION_RULE_VERSION {
            return Err("unsupported OrientedSubcontinuationPlanV1 representation rule version");
        }
        let count = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| "OrientedSubcontinuationPlanV1 frame count overflows usize")?;
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            let frame_id = take_u64(&mut bytes)?;
            let segment_site_id = take_u64(&mut bytes)?;
            let declaration = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "oriented subcontinuation declaration is not UTF-8")?;
            let path_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "oriented subcontinuation occurrence path overflows usize")?;
            let mut checked_occurrence_path = Vec::with_capacity(path_len);
            for _ in 0..path_len {
                checked_occurrence_path.push(take_u64(&mut bytes)?);
            }
            let semantic_position = take_u64(&mut bytes)?;
            let input_interface =
                CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let output_interface =
                CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let runtime_frame_fingerprint = take_u64(&mut bytes)?;
            let occurrence_binding_fingerprint = take_u64(&mut bytes)?;
            let (tag, tail) = bytes
                .split_first()
                .ok_or("truncated oriented subcontinuation control witness")?;
            bytes = tail;
            let control_witness = match tag {
                0 => OrientedControlWitnessV1::DistinguishedRoot,
                1 => OrientedControlWitnessV1::ParentFrame(take_u64(&mut bytes)?),
                _ => return Err("unknown oriented subcontinuation control witness"),
            };
            frames.push(OrientedSubcontinuationFramePlanV1 {
                frame_id,
                segment_site_id,
                declaration,
                checked_occurrence_path,
                semantic_position,
                input_interface,
                output_interface,
                runtime_frame_fingerprint,
                occurrence_binding_fingerprint,
                control_witness,
            });
        }
        if !bytes.is_empty() {
            return Err("OrientedSubcontinuationPlanV1 has trailing bytes");
        }
        let plan = Self {
            representation_rule_version,
            frames,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        let mut frame_ids = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for frame in &self.frames {
            if !frame_ids.insert(frame.frame_id) {
                return Err("OrientedSubcontinuationPlanV1 repeats a frame identity");
            }
            if !positions.insert(frame.semantic_position) {
                return Err("OrientedSubcontinuationPlanV1 repeats a semantic position");
            }
            if frame.occurrence_binding_fingerprint
                != compiler_private_oriented_occurrence_binding_fingerprint(frame)
            {
                return Err("oriented subcontinuation occurrence binding is inconsistent");
            }
        }
        let by_id = self
            .frames
            .iter()
            .map(|frame| (frame.frame_id, frame))
            .collect::<BTreeMap<_, _>>();
        let mut roots_by_segment = BTreeMap::<u64, usize>::new();
        for frame in &self.frames {
            if let OrientedControlWitnessV1::ParentFrame(parent) = frame.control_witness {
                if parent == frame.frame_id || !by_id.contains_key(&parent) {
                    return Err("oriented subcontinuation parent witness is stale");
                }
                if by_id[&parent].segment_site_id != frame.segment_site_id {
                    return Err("oriented subcontinuation parent crosses a prompt region");
                }
                let mut seen = BTreeSet::from([frame.frame_id]);
                let mut cursor = parent;
                loop {
                    if !seen.insert(cursor) {
                        return Err("oriented subcontinuation parent witnesses form a cycle");
                    }
                    match by_id
                        .get(&cursor)
                        .ok_or("oriented subcontinuation parent witness is stale")?
                        .control_witness
                    {
                        OrientedControlWitnessV1::DistinguishedRoot => break,
                        OrientedControlWitnessV1::ParentFrame(next) => cursor = next,
                    }
                }
            } else {
                *roots_by_segment.entry(frame.segment_site_id).or_default() += 1;
            }
        }
        let mut by_segment = BTreeMap::<u64, Vec<&OrientedSubcontinuationFramePlanV1>>::new();
        for frame in &self.frames {
            by_segment.entry(frame.segment_site_id).or_default().push(frame);
        }
        for (segment, frames) in &mut by_segment {
            if roots_by_segment.get(segment).copied() != Some(1) {
                return Err("oriented subcontinuation prompt region has no unique root");
            }
            frames.sort_by_key(|frame| frame.semantic_position);
            for pair in frames.windows(2) {
                if pair[0].output_interface != pair[1].input_interface {
                    return Err("oriented subcontinuation checked endpoints do not compose");
                }
            }
        }
        Ok(())
    }

    pub fn transport_hash(&self) -> u64 {
        fnv1a_64(&self.canonical_bytes())
    }

    pub fn frame(&self, frame_id: u64) -> Option<&OrientedSubcontinuationFramePlanV1> {
        self.frames.iter().find(|frame| frame.frame_id == frame_id)
    }
}

#[doc(hidden)]
pub fn compiler_private_oriented_occurrence_binding_fingerprint(
    frame: &OrientedSubcontinuationFramePlanV1,
) -> u64 {
    let mut bytes = b"OrientedSubcontinuationOccurrenceV1\0".to_vec();
    put_u64(&mut bytes, frame.frame_id);
    put_u64(&mut bytes, frame.segment_site_id);
    put_bytes(&mut bytes, frame.declaration.as_bytes());
    put_u64(&mut bytes, frame.checked_occurrence_path.len() as u64);
    for value in &frame.checked_occurrence_path {
        put_u64(&mut bytes, *value);
    }
    put_u64(&mut bytes, frame.semantic_position);
    put_bytes(&mut bytes, &frame.input_interface.canonical);
    put_bytes(&mut bytes, &frame.output_interface.canonical);
    put_u64(&mut bytes, frame.runtime_frame_fingerprint);
    match frame.control_witness {
        OrientedControlWitnessV1::DistinguishedRoot => bytes.push(0),
        OrientedControlWitnessV1::ParentFrame(parent) => {
            bytes.push(1);
            put_u64(&mut bytes, parent);
        }
    }
    fnv1a_64(&bytes)
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn take_u32(bytes: &mut &[u8]) -> Result<u32, &'static str> {
    let (head, tail) = bytes
        .split_at_checked(4)
        .ok_or("truncated oriented subcontinuation u32")?;
    *bytes = tail;
    Ok(u32::from_be_bytes(head.try_into().expect("exact width")))
}

fn take_u64(bytes: &mut &[u8]) -> Result<u64, &'static str> {
    let (head, tail) = bytes
        .split_at_checked(8)
        .ok_or("truncated oriented subcontinuation u64")?;
    *bytes = tail;
    Ok(u64::from_be_bytes(head.try_into().expect("exact width")))
}

fn take_bytes<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let len = usize::try_from(take_u64(bytes)?)
        .map_err(|_| "oriented subcontinuation byte length overflows usize")?;
    let (head, tail) = bytes
        .split_at_checked(len)
        .ok_or("truncated oriented subcontinuation bytes")?;
    *bytes = tail;
    Ok(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interface(name: &[u8]) -> CheckedAnswerInterfaceV1 {
        let mut canonical = CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
        canonical.extend_from_slice(name);
        CheckedAnswerInterfaceV1::new(canonical).unwrap()
    }

    fn frame(id: u64, parent: Option<u64>) -> OrientedSubcontinuationFramePlanV1 {
        let mut frame = OrientedSubcontinuationFramePlanV1 {
            frame_id: id,
            segment_site_id: 10,
            declaration: "pkg::main".to_string(),
            checked_occurrence_path: vec![id],
            semantic_position: id,
            input_interface: interface(&[id as u8]),
            output_interface: interface(&[id as u8 + 1]),
            runtime_frame_fingerprint: id + 20,
            occurrence_binding_fingerprint: 0,
            control_witness: parent.map_or(
                OrientedControlWitnessV1::DistinguishedRoot,
                OrientedControlWitnessV1::ParentFrame,
            ),
        };
        frame.occurrence_binding_fingerprint =
            compiler_private_oriented_occurrence_binding_fingerprint(&frame);
        frame
    }

    #[test]
    fn canonical_roundtrip_keeps_full_checked_interfaces() {
        let plan = OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame(0, None), frame(1, Some(0))],
        };
        assert_eq!(
            OrientedSubcontinuationPlanV1::decode(&plan.canonical_bytes()).unwrap(),
            plan
        );
    }

    #[test]
    fn endpoint_or_occurrence_corruption_rejects() {
        let mut frame = frame(0, None);
        frame.output_interface.canonical.push(9);
        let plan = OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame],
        };
        assert_eq!(
            OrientedSubcontinuationPlanV1::decode(&plan.canonical_bytes()),
            Err("oriented subcontinuation occurrence binding is inconsistent")
        );
    }

    #[test]
    fn occurrence_exact_but_semantic_order_reversed_rejects() {
        let mut outer = frame(0, None);
        let mut inner = frame(1, Some(0));
        outer.semantic_position = 1;
        inner.semantic_position = 0;
        outer.occurrence_binding_fingerprint =
            compiler_private_oriented_occurrence_binding_fingerprint(&outer);
        inner.occurrence_binding_fingerprint =
            compiler_private_oriented_occurrence_binding_fingerprint(&inner);
        let plan = OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![outer, inner],
        };
        assert_eq!(
            plan.validate(),
            Err("oriented subcontinuation checked endpoints do not compose")
        );
    }

    #[test]
    fn control_parent_cannot_cross_prompt_regions() {
        let root = frame(0, None);
        let mut child = frame(1, Some(0));
        child.segment_site_id = 99;
        child.occurrence_binding_fingerprint =
            compiler_private_oriented_occurrence_binding_fingerprint(&child);
        let plan = OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![root, child],
        };
        assert_eq!(
            plan.validate(),
            Err("oriented subcontinuation parent crosses a prompt region")
        );
    }
}
