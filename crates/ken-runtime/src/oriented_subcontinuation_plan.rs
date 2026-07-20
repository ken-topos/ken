//! Compiler-private checked oriented-subcontinuation plan transport.

use std::collections::{BTreeMap, BTreeSet};

use crate::fnv1a_64;

pub const ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER: &[u8] = b"OrientedSubcontinuationPlanV1\0";
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

/// One reusable checked edge at a complete same-SCC application occurrence.
/// The template is static; native lowering mints a fresh affine invocation
/// identity each time it consumes the matching Runtime marker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedRecursiveInvocationTemplateV1 {
    pub call_template_id: u64,
    pub declaration: String,
    pub checked_occurrence_path: Vec<u64>,
    pub callee: String,
    pub level_instantiation: Vec<Vec<u8>>,
    pub recursion_group: String,
    pub scc_index: u64,
    pub admission: u8,
    pub arity: u64,
    pub local_telescope: Vec<CheckedAnswerInterfaceV1>,
    pub result_interface: CheckedAnswerInterfaceV1,
    pub callee_segment_site_id: u64,
    pub callee_frame_templates: Vec<u64>,
    pub caller_interface: CheckedAnswerInterfaceV1,
    pub occurrence_binding_fingerprint: u64,
}

/// One checked recursive-hypothesis binder introduced by an exact
/// computational eliminator branch.  This is a reusable static template, not
/// a dynamic invocation identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedComputationalIHSlotTemplateV1 {
    pub slot_template_id: u64,
    pub declaration: String,
    pub checked_match_ordinal: u64,
    pub checked_occurrence_path: Vec<u64>,
    pub frame_template_id: u64,
    pub constructor: String,
    pub recursive_position: u64,
    pub method_binder_ordinal: u64,
    pub local_telescope: Vec<CheckedAnswerInterfaceV1>,
    pub ih_interface: CheckedAnswerInterfaceV1,
    pub segment_site_id: u64,
    pub frame_templates: Vec<u64>,
    pub input_interface: CheckedAnswerInterfaceV1,
    pub output_interface: CheckedAnswerInterfaceV1,
    pub occurrence_binding_fingerprint: u64,
}

/// One exact complete application occurrence of a checked computational IH.
/// Native lowering mints a fresh affine dynamic identity only while consuming
/// the matching Runtime marker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedComputationalIHCallTemplateV1 {
    pub call_template_id: u64,
    pub declaration: String,
    pub checked_occurrence_path: Vec<u64>,
    pub slot_template_id: u64,
    pub arity: u64,
    pub local_telescope: Vec<CheckedAnswerInterfaceV1>,
    pub result_interface: CheckedAnswerInterfaceV1,
    pub callee_segment_site_id: u64,
    pub callee_frame_templates: Vec<u64>,
    pub parent_frame_template_id: Option<u64>,
    pub parent_segment_site_id: Option<u64>,
    pub caller_interface: CheckedAnswerInterfaceV1,
    pub occurrence_binding_fingerprint: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientedSubcontinuationPlanV1 {
    pub representation_rule_version: u32,
    pub frames: Vec<OrientedSubcontinuationFramePlanV1>,
    pub recursive_calls: Vec<CheckedRecursiveInvocationTemplateV1>,
    pub computational_ih_slots: Vec<CheckedComputationalIHSlotTemplateV1>,
    pub computational_ih_calls: Vec<CheckedComputationalIHCallTemplateV1>,
}

impl OrientedSubcontinuationPlanV1 {
    pub const REPRESENTATION_RULE_VERSION: u32 = 3;

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
        put_u64(&mut out, self.recursive_calls.len() as u64);
        for call in &self.recursive_calls {
            put_u64(&mut out, call.call_template_id);
            put_bytes(&mut out, call.declaration.as_bytes());
            put_u64(&mut out, call.checked_occurrence_path.len() as u64);
            for value in &call.checked_occurrence_path {
                put_u64(&mut out, *value);
            }
            put_bytes(&mut out, call.callee.as_bytes());
            put_u64(&mut out, call.level_instantiation.len() as u64);
            for level in &call.level_instantiation {
                put_bytes(&mut out, level);
            }
            put_bytes(&mut out, call.recursion_group.as_bytes());
            put_u64(&mut out, call.scc_index);
            out.push(call.admission);
            put_u64(&mut out, call.arity);
            put_u64(&mut out, call.local_telescope.len() as u64);
            for entry in &call.local_telescope {
                put_bytes(&mut out, &entry.canonical);
            }
            put_bytes(&mut out, &call.result_interface.canonical);
            put_u64(&mut out, call.callee_segment_site_id);
            put_u64(&mut out, call.callee_frame_templates.len() as u64);
            for frame in &call.callee_frame_templates {
                put_u64(&mut out, *frame);
            }
            put_bytes(&mut out, &call.caller_interface.canonical);
            put_u64(&mut out, call.occurrence_binding_fingerprint);
        }
        put_u64(&mut out, self.computational_ih_slots.len() as u64);
        for slot in &self.computational_ih_slots {
            put_u64(&mut out, slot.slot_template_id);
            put_bytes(&mut out, slot.declaration.as_bytes());
            put_u64(&mut out, slot.checked_match_ordinal);
            put_u64(&mut out, slot.checked_occurrence_path.len() as u64);
            for value in &slot.checked_occurrence_path {
                put_u64(&mut out, *value);
            }
            put_u64(&mut out, slot.frame_template_id);
            put_bytes(&mut out, slot.constructor.as_bytes());
            put_u64(&mut out, slot.recursive_position);
            put_u64(&mut out, slot.method_binder_ordinal);
            put_u64(&mut out, slot.local_telescope.len() as u64);
            for entry in &slot.local_telescope {
                put_bytes(&mut out, &entry.canonical);
            }
            put_bytes(&mut out, &slot.ih_interface.canonical);
            put_u64(&mut out, slot.segment_site_id);
            put_u64(&mut out, slot.frame_templates.len() as u64);
            for frame in &slot.frame_templates {
                put_u64(&mut out, *frame);
            }
            put_bytes(&mut out, &slot.input_interface.canonical);
            put_bytes(&mut out, &slot.output_interface.canonical);
            put_u64(&mut out, slot.occurrence_binding_fingerprint);
        }
        put_u64(&mut out, self.computational_ih_calls.len() as u64);
        for call in &self.computational_ih_calls {
            put_u64(&mut out, call.call_template_id);
            put_bytes(&mut out, call.declaration.as_bytes());
            put_u64(&mut out, call.checked_occurrence_path.len() as u64);
            for value in &call.checked_occurrence_path {
                put_u64(&mut out, *value);
            }
            put_u64(&mut out, call.slot_template_id);
            put_u64(&mut out, call.arity);
            put_u64(&mut out, call.local_telescope.len() as u64);
            for entry in &call.local_telescope {
                put_bytes(&mut out, &entry.canonical);
            }
            put_bytes(&mut out, &call.result_interface.canonical);
            put_u64(&mut out, call.callee_segment_site_id);
            put_u64(&mut out, call.callee_frame_templates.len() as u64);
            for frame in &call.callee_frame_templates {
                put_u64(&mut out, *frame);
            }
            put_optional_u64(&mut out, call.parent_frame_template_id);
            put_optional_u64(&mut out, call.parent_segment_site_id);
            put_bytes(&mut out, &call.caller_interface.canonical);
            put_u64(&mut out, call.occurrence_binding_fingerprint);
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        let Some(mut bytes) = bytes.strip_prefix(ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER) else {
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
            let input_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let output_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
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
        let call_count = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| "recursive invocation template count overflows usize")?;
        let mut recursive_calls = Vec::with_capacity(call_count);
        for _ in 0..call_count {
            let call_template_id = take_u64(&mut bytes)?;
            let declaration = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "recursive invocation declaration is not UTF-8")?;
            let path_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "recursive invocation occurrence path overflows usize")?;
            let mut checked_occurrence_path = Vec::with_capacity(path_len);
            for _ in 0..path_len {
                checked_occurrence_path.push(take_u64(&mut bytes)?);
            }
            let callee = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "recursive invocation callee is not UTF-8")?;
            let level_count = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "recursive invocation level count overflows usize")?;
            let mut level_instantiation = Vec::with_capacity(level_count);
            for _ in 0..level_count {
                level_instantiation.push(take_bytes(&mut bytes)?.to_vec());
            }
            let recursion_group = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "recursive invocation group is not UTF-8")?;
            let scc_index = take_u64(&mut bytes)?;
            let (admission, tail) = bytes
                .split_first()
                .ok_or("truncated recursive invocation admission")?;
            bytes = tail;
            let admission = *admission;
            let arity = take_u64(&mut bytes)?;
            let telescope_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "recursive invocation telescope length overflows usize")?;
            let mut local_telescope = Vec::with_capacity(telescope_len);
            for _ in 0..telescope_len {
                local_telescope.push(CheckedAnswerInterfaceV1::new(
                    take_bytes(&mut bytes)?.to_vec(),
                )?);
            }
            let result_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let callee_segment_site_id = take_u64(&mut bytes)?;
            let frame_count = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "recursive invocation frame count overflows usize")?;
            let mut callee_frame_templates = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                callee_frame_templates.push(take_u64(&mut bytes)?);
            }
            let caller_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let occurrence_binding_fingerprint = take_u64(&mut bytes)?;
            recursive_calls.push(CheckedRecursiveInvocationTemplateV1 {
                call_template_id,
                declaration,
                checked_occurrence_path,
                callee,
                level_instantiation,
                recursion_group,
                scc_index,
                admission,
                arity,
                local_telescope,
                result_interface,
                callee_segment_site_id,
                callee_frame_templates,
                caller_interface,
                occurrence_binding_fingerprint,
            });
        }
        let slot_count = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| "computational IH slot count overflows usize")?;
        let mut computational_ih_slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            let slot_template_id = take_u64(&mut bytes)?;
            let declaration = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "computational IH slot declaration is not UTF-8")?;
            let checked_match_ordinal = take_u64(&mut bytes)?;
            let path_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH slot occurrence path overflows usize")?;
            let mut checked_occurrence_path = Vec::with_capacity(path_len);
            for _ in 0..path_len {
                checked_occurrence_path.push(take_u64(&mut bytes)?);
            }
            let frame_template_id = take_u64(&mut bytes)?;
            let constructor = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "computational IH slot constructor is not UTF-8")?;
            let recursive_position = take_u64(&mut bytes)?;
            let method_binder_ordinal = take_u64(&mut bytes)?;
            let telescope_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH slot telescope length overflows usize")?;
            let mut local_telescope = Vec::with_capacity(telescope_len);
            for _ in 0..telescope_len {
                local_telescope.push(CheckedAnswerInterfaceV1::new(
                    take_bytes(&mut bytes)?.to_vec(),
                )?);
            }
            let ih_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let segment_site_id = take_u64(&mut bytes)?;
            let frame_count = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH slot frame count overflows usize")?;
            let mut frame_templates = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                frame_templates.push(take_u64(&mut bytes)?);
            }
            let input_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let output_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let occurrence_binding_fingerprint = take_u64(&mut bytes)?;
            computational_ih_slots.push(CheckedComputationalIHSlotTemplateV1 {
                slot_template_id,
                declaration,
                checked_match_ordinal,
                checked_occurrence_path,
                frame_template_id,
                constructor,
                recursive_position,
                method_binder_ordinal,
                local_telescope,
                ih_interface,
                segment_site_id,
                frame_templates,
                input_interface,
                output_interface,
                occurrence_binding_fingerprint,
            });
        }
        let ih_call_count = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| "computational IH call count overflows usize")?;
        let mut computational_ih_calls = Vec::with_capacity(ih_call_count);
        for _ in 0..ih_call_count {
            let call_template_id = take_u64(&mut bytes)?;
            let declaration = String::from_utf8(take_bytes(&mut bytes)?.to_vec())
                .map_err(|_| "computational IH call declaration is not UTF-8")?;
            let path_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH call occurrence path overflows usize")?;
            let mut checked_occurrence_path = Vec::with_capacity(path_len);
            for _ in 0..path_len {
                checked_occurrence_path.push(take_u64(&mut bytes)?);
            }
            let slot_template_id = take_u64(&mut bytes)?;
            let arity = take_u64(&mut bytes)?;
            let telescope_len = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH call telescope length overflows usize")?;
            let mut local_telescope = Vec::with_capacity(telescope_len);
            for _ in 0..telescope_len {
                local_telescope.push(CheckedAnswerInterfaceV1::new(
                    take_bytes(&mut bytes)?.to_vec(),
                )?);
            }
            let result_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let callee_segment_site_id = take_u64(&mut bytes)?;
            let frame_count = usize::try_from(take_u64(&mut bytes)?)
                .map_err(|_| "computational IH call frame count overflows usize")?;
            let mut callee_frame_templates = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                callee_frame_templates.push(take_u64(&mut bytes)?);
            }
            let parent_frame_template_id = take_optional_u64(&mut bytes)?;
            let parent_segment_site_id = take_optional_u64(&mut bytes)?;
            let caller_interface = CheckedAnswerInterfaceV1::new(take_bytes(&mut bytes)?.to_vec())?;
            let occurrence_binding_fingerprint = take_u64(&mut bytes)?;
            computational_ih_calls.push(CheckedComputationalIHCallTemplateV1 {
                call_template_id,
                declaration,
                checked_occurrence_path,
                slot_template_id,
                arity,
                local_telescope,
                result_interface,
                callee_segment_site_id,
                callee_frame_templates,
                parent_frame_template_id,
                parent_segment_site_id,
                caller_interface,
                occurrence_binding_fingerprint,
            });
        }
        if !bytes.is_empty() {
            return Err("OrientedSubcontinuationPlanV1 has trailing bytes");
        }
        let plan = Self {
            representation_rule_version,
            frames,
            recursive_calls,
            computational_ih_slots,
            computational_ih_calls,
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
        let mut call_ids = BTreeSet::new();
        for call in &self.recursive_calls {
            if !call_ids.insert(call.call_template_id) {
                return Err("OrientedSubcontinuationPlanV1 repeats a recursive call template");
            }
            if call.arity == 0 || call.callee_frame_templates.is_empty() {
                return Err("recursive call template is partial or has no callee segment");
            }
            if call.occurrence_binding_fingerprint
                != compiler_private_recursive_call_binding_fingerprint(call)
            {
                return Err("recursive call template occurrence binding is inconsistent");
            }
            for frame_id in &call.callee_frame_templates {
                let frame = self
                    .frames
                    .iter()
                    .find(|frame| frame.frame_id == *frame_id)
                    .ok_or("recursive call template names a stale callee frame")?;
                if frame.segment_site_id != call.callee_segment_site_id
                    || frame.declaration != call.callee
                {
                    return Err("recursive call template callee binding is inconsistent");
                }
            }
            let mut callee_frames = call
                .callee_frame_templates
                .iter()
                .map(|id| {
                    self.frames
                        .iter()
                        .find(|frame| frame.frame_id == *id)
                        .unwrap()
                })
                .collect::<Vec<_>>();
            callee_frames.sort_by_key(|frame| frame.semantic_position);
            if callee_frames.last().unwrap().output_interface != call.result_interface
                || call.result_interface != call.caller_interface
            {
                return Err("recursive call template checked endpoints do not compose");
            }
        }
        let mut slot_ids = BTreeSet::new();
        for slot in &self.computational_ih_slots {
            if !slot_ids.insert(slot.slot_template_id) {
                return Err("OrientedSubcontinuationPlanV1 repeats a computational IH slot");
            }
            if slot.frame_templates.is_empty()
                || !slot.frame_templates.contains(&slot.frame_template_id)
            {
                return Err("computational IH slot has no exact frame template");
            }
            if slot.occurrence_binding_fingerprint
                != compiler_private_computational_ih_slot_binding_fingerprint(slot)
            {
                return Err("computational IH slot occurrence binding is inconsistent");
            }
            for frame_id in &slot.frame_templates {
                let frame = self
                    .frame(*frame_id)
                    .ok_or("computational IH slot names a stale frame template")?;
                if frame.segment_site_id != slot.segment_site_id {
                    return Err("computational IH slot crosses a checked segment");
                }
            }
        }
        let mut ih_call_ids = BTreeSet::new();
        for call in &self.computational_ih_calls {
            if !ih_call_ids.insert(call.call_template_id) {
                return Err("OrientedSubcontinuationPlanV1 repeats a computational IH call");
            }
            let slot = self
                .computational_ih_slot(call.slot_template_id)
                .ok_or("computational IH call names a stale slot template")?;
            if call.callee_frame_templates.is_empty()
                || call.callee_segment_site_id != slot.segment_site_id
                || call.callee_frame_templates != slot.frame_templates
                || call.occurrence_binding_fingerprint
                    != compiler_private_computational_ih_call_binding_fingerprint(call)
            {
                return Err("computational IH call binding is inconsistent");
            }
            if call
                .callee_frame_templates
                .iter()
                .any(|frame| self.frame(*frame).is_none())
            {
                return Err("computational IH call names a stale callee frame");
            }
            match (call.parent_frame_template_id, call.parent_segment_site_id) {
                (Some(parent_id), Some(parent_segment)) => {
                    let parent = self
                        .frame(parent_id)
                        .ok_or("computational IH call names a stale parent frame")?;
                    if parent.segment_site_id != parent_segment
                        || parent_segment != call.callee_segment_site_id
                        || parent_id != slot.frame_template_id
                    {
                        return Err("computational IH call parent edge is inconsistent");
                    }
                    if call.result_interface != call.caller_interface {
                        return Err("computational IH call checked endpoints do not compose");
                    }
                }
                (None, None) => return Err("computational IH call parent edge is missing"),
                _ => return Err("computational IH call parent edge is incomplete"),
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
            by_segment
                .entry(frame.segment_site_id)
                .or_default()
                .push(frame);
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

    pub fn recursive_call(
        &self,
        call_template_id: u64,
    ) -> Option<&CheckedRecursiveInvocationTemplateV1> {
        self.recursive_calls
            .iter()
            .find(|call| call.call_template_id == call_template_id)
    }

    pub fn computational_ih_slot(
        &self,
        slot_template_id: u64,
    ) -> Option<&CheckedComputationalIHSlotTemplateV1> {
        self.computational_ih_slots
            .iter()
            .find(|slot| slot.slot_template_id == slot_template_id)
    }

    pub fn computational_ih_call(
        &self,
        call_template_id: u64,
    ) -> Option<&CheckedComputationalIHCallTemplateV1> {
        self.computational_ih_calls
            .iter()
            .find(|call| call.call_template_id == call_template_id)
    }
}

#[doc(hidden)]
pub fn compiler_private_computational_ih_slot_binding_fingerprint(
    slot: &CheckedComputationalIHSlotTemplateV1,
) -> u64 {
    let mut bytes = b"CheckedComputationalIHSlotOccurrenceV1\0".to_vec();
    put_u64(&mut bytes, slot.slot_template_id);
    put_bytes(&mut bytes, slot.declaration.as_bytes());
    put_u64(&mut bytes, slot.checked_match_ordinal);
    for value in &slot.checked_occurrence_path {
        put_u64(&mut bytes, *value);
    }
    put_u64(&mut bytes, slot.frame_template_id);
    put_bytes(&mut bytes, slot.constructor.as_bytes());
    put_u64(&mut bytes, slot.recursive_position);
    put_u64(&mut bytes, slot.method_binder_ordinal);
    for entry in &slot.local_telescope {
        put_bytes(&mut bytes, &entry.canonical);
    }
    put_bytes(&mut bytes, &slot.ih_interface.canonical);
    put_u64(&mut bytes, slot.segment_site_id);
    for frame in &slot.frame_templates {
        put_u64(&mut bytes, *frame);
    }
    put_bytes(&mut bytes, &slot.input_interface.canonical);
    put_bytes(&mut bytes, &slot.output_interface.canonical);
    fnv1a_64(&bytes)
}

#[doc(hidden)]
pub fn compiler_private_computational_ih_call_binding_fingerprint(
    call: &CheckedComputationalIHCallTemplateV1,
) -> u64 {
    let mut bytes = b"CheckedComputationalIHCallOccurrenceV1\0".to_vec();
    put_u64(&mut bytes, call.call_template_id);
    put_bytes(&mut bytes, call.declaration.as_bytes());
    for value in &call.checked_occurrence_path {
        put_u64(&mut bytes, *value);
    }
    put_u64(&mut bytes, call.slot_template_id);
    put_u64(&mut bytes, call.arity);
    for entry in &call.local_telescope {
        put_bytes(&mut bytes, &entry.canonical);
    }
    put_bytes(&mut bytes, &call.result_interface.canonical);
    put_u64(&mut bytes, call.callee_segment_site_id);
    for frame in &call.callee_frame_templates {
        put_u64(&mut bytes, *frame);
    }
    put_optional_u64(&mut bytes, call.parent_frame_template_id);
    put_optional_u64(&mut bytes, call.parent_segment_site_id);
    put_bytes(&mut bytes, &call.caller_interface.canonical);
    fnv1a_64(&bytes)
}

#[doc(hidden)]
pub fn compiler_private_recursive_call_binding_fingerprint(
    call: &CheckedRecursiveInvocationTemplateV1,
) -> u64 {
    let mut bytes = b"CheckedRecursiveInvocationOccurrenceV1\0".to_vec();
    put_u64(&mut bytes, call.call_template_id);
    put_bytes(&mut bytes, call.declaration.as_bytes());
    put_u64(&mut bytes, call.checked_occurrence_path.len() as u64);
    for value in &call.checked_occurrence_path {
        put_u64(&mut bytes, *value);
    }
    put_bytes(&mut bytes, call.callee.as_bytes());
    for level in &call.level_instantiation {
        put_bytes(&mut bytes, level);
    }
    put_bytes(&mut bytes, call.recursion_group.as_bytes());
    put_u64(&mut bytes, call.scc_index);
    bytes.push(call.admission);
    put_u64(&mut bytes, call.arity);
    for entry in &call.local_telescope {
        put_bytes(&mut bytes, &entry.canonical);
    }
    put_bytes(&mut bytes, &call.result_interface.canonical);
    put_u64(&mut bytes, call.callee_segment_site_id);
    for frame in &call.callee_frame_templates {
        put_u64(&mut bytes, *frame);
    }
    put_bytes(&mut bytes, &call.caller_interface.canonical);
    fnv1a_64(&bytes)
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

fn put_optional_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        None => out.push(0),
        Some(value) => {
            out.push(1);
            put_u64(out, value);
        }
    }
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

fn take_optional_u64(bytes: &mut &[u8]) -> Result<Option<u64>, &'static str> {
    let (tag, tail) = bytes
        .split_first()
        .ok_or("truncated oriented subcontinuation optional u64")?;
    *bytes = tail;
    match tag {
        0 => Ok(None),
        1 => Ok(Some(take_u64(bytes)?)),
        _ => Err("invalid oriented subcontinuation optional u64 tag"),
    }
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

    fn computational_ih_plan() -> OrientedSubcontinuationPlanV1 {
        let frames = vec![frame(0, None), frame(1, Some(0))];
        let mut slots = Vec::new();
        let mut calls = Vec::new();
        for frame_id in 0..=1 {
            let slot_template_id = 20 + frame_id;
            let mut slot = CheckedComputationalIHSlotTemplateV1 {
                slot_template_id,
                declaration: "pkg::main".to_string(),
                checked_match_ordinal: frame_id,
                checked_occurrence_path: vec![2, frame_id],
                frame_template_id: frame_id,
                constructor: format!("Ctor{frame_id}"),
                recursive_position: 0,
                method_binder_ordinal: 0,
                local_telescope: Vec::new(),
                ih_interface: interface(&[frame_id as u8]),
                segment_site_id: 10,
                frame_templates: vec![frame_id],
                input_interface: interface(&[frame_id as u8]),
                output_interface: interface(&[frame_id as u8 + 1]),
                occurrence_binding_fingerprint: 0,
            };
            slot.occurrence_binding_fingerprint =
                compiler_private_computational_ih_slot_binding_fingerprint(&slot);
            slots.push(slot);

            let mut call = CheckedComputationalIHCallTemplateV1 {
                call_template_id: 30 + frame_id,
                declaration: "pkg::main".to_string(),
                checked_occurrence_path: vec![3, frame_id],
                slot_template_id,
                arity: 1,
                local_telescope: Vec::new(),
                result_interface: interface(&[frame_id as u8 + 1]),
                callee_segment_site_id: 10,
                callee_frame_templates: vec![frame_id],
                parent_frame_template_id: Some(frame_id),
                parent_segment_site_id: Some(10),
                caller_interface: interface(&[frame_id as u8 + 1]),
                occurrence_binding_fingerprint: 0,
            };
            call.occurrence_binding_fingerprint =
                compiler_private_computational_ih_call_binding_fingerprint(&call);
            calls.push(call);
        }
        OrientedSubcontinuationPlanV1 {
            representation_rule_version: OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames,
            recursive_calls: Vec::new(),
            computational_ih_slots: slots,
            computational_ih_calls: calls,
        }
    }

    #[test]
    fn canonical_roundtrip_keeps_full_checked_interfaces() {
        let plan = OrientedSubcontinuationPlanV1 {
            representation_rule_version: OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame(0, None), frame(1, Some(0))],
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
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
            representation_rule_version: OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame],
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
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
            representation_rule_version: OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![outer, inner],
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
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
            representation_rule_version: OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![root, child],
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
        };
        assert_eq!(
            plan.validate(),
            Err("oriented subcontinuation parent crosses a prompt region")
        );
    }

    #[test]
    fn computational_ih_parent_edges_roundtrip_and_bind_exact_occurrences() {
        let plan = computational_ih_plan();
        plan.validate().unwrap();
        assert_eq!(
            OrientedSubcontinuationPlanV1::decode(&plan.canonical_bytes()).unwrap(),
            plan
        );
    }

    #[test]
    fn computational_ih_parent_edge_deletion_corruption_and_swap_reject() {
        let exact = computational_ih_plan();

        let mut deleted = exact.clone();
        deleted.computational_ih_calls[0].parent_frame_template_id = None;
        deleted.computational_ih_calls[0].parent_segment_site_id = None;
        deleted.computational_ih_calls[0].occurrence_binding_fingerprint =
            compiler_private_computational_ih_call_binding_fingerprint(
                &deleted.computational_ih_calls[0],
            );
        assert_eq!(
            deleted.validate(),
            Err("computational IH call parent edge is missing")
        );

        let mut incomplete = exact.clone();
        incomplete.computational_ih_calls[0].parent_segment_site_id = None;
        incomplete.computational_ih_calls[0].occurrence_binding_fingerprint =
            compiler_private_computational_ih_call_binding_fingerprint(
                &incomplete.computational_ih_calls[0],
            );
        assert_eq!(
            incomplete.validate(),
            Err("computational IH call parent edge is incomplete")
        );

        let mut transplanted = exact.clone();
        transplanted.computational_ih_calls[0].parent_segment_site_id = Some(99);
        transplanted.computational_ih_calls[0].occurrence_binding_fingerprint =
            compiler_private_computational_ih_call_binding_fingerprint(
                &transplanted.computational_ih_calls[0],
            );
        assert_eq!(
            transplanted.validate(),
            Err("computational IH call parent edge is inconsistent")
        );

        let mut swapped = exact;
        swapped.computational_ih_calls[0].parent_frame_template_id = Some(1);
        swapped.computational_ih_calls[1].parent_frame_template_id = Some(0);
        for call in &mut swapped.computational_ih_calls {
            call.occurrence_binding_fingerprint =
                compiler_private_computational_ih_call_binding_fingerprint(call);
        }
        assert_eq!(
            swapped.validate(),
            Err("computational IH call parent edge is inconsistent")
        );
    }
}
