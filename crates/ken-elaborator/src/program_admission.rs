//! Checked admission for the single supported Ken program entrypoint ABI.

use ken_kernel::{Context, Decl, GlobalId, Term};

use crate::{capabilities, effects::EffectRow, ElabEnv};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedMainDescriptor {
    pub main: GlobalId,
    pub process_input: GlobalId,
    pub process_input_constructor: GlobalId,
    pub program_caps: GlobalId,
    pub program_caps_constructor: GlobalId,
    pub cap: GlobalId,
    pub authority_constructor: GlobalId,
    pub host_io: GlobalId,
    pub host_exit: GlobalId,
    pub exit_code: GlobalId,
    pub success_constructor: GlobalId,
    pub failure_constructor: GlobalId,
    pub ret_constructor: GlobalId,
    pub list_nil_constructor: GlobalId,
    pub list_cons_constructor: GlobalId,
    pub prod_constructor: GlobalId,
    pub authority_name: String,
    pub authority: capabilities::Authority,
    pub allow_root_execution: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgramAdmissionError {
    MissingMain,
    MissingProgramBoundary,
    MissingAbiDeclaration { name: &'static str },
    MissingCapability { effect: &'static str },
    DuplicateCapability { effect: &'static str },
    UnsupportedEffectRow,
    InvalidMainAbi { authority: String },
}

/// Admit the one process-shaped Program I entrypoint against the live
/// elaboration environment that produced it.
pub fn admit_checked_main(env: &ElabEnv) -> Result<CheckedMainDescriptor, ProgramAdmissionError> {
    let get = |name: &'static str| {
        env.globals
            .get(name)
            .copied()
            .ok_or(ProgramAdmissionError::MissingAbiDeclaration { name })
    };
    let main = env
        .globals
        .get("main")
        .copied()
        .ok_or(ProgramAdmissionError::MissingMain)?;

    let header = env
        .boundary_header()
        .filter(|header| header.kind == crate::BoundaryKind::Program)
        .ok_or(ProgramAdmissionError::MissingProgramBoundary)?;
    let capabilities = header
        .capabilities
        .as_ref()
        .ok_or(ProgramAdmissionError::MissingCapability { effect: "FS" })?;
    let fs = capabilities
        .iter()
        .filter(|capability| capability.family == "FS")
        .collect::<Vec<_>>();
    let declaration = match fs.as_slice() {
        [] => return Err(ProgramAdmissionError::MissingCapability { effect: "FS" }),
        [declaration] => *declaration,
        _ => return Err(ProgramAdmissionError::DuplicateCapability { effect: "FS" }),
    };
    let (authority_name, authority) = match declaration.authority.as_str() {
        "ANone" => ("ANone", capabilities::AUTH_NONE),
        "APartial" => ("APartial", capabilities::AUTH_PARTIAL),
        "AFull" => ("AFull", capabilities::AUTH_FULL),
        other => unreachable!("parser admitted invalid FS authority {other}"),
    };

    if let Some(row) = env.effect_rows.get("main") {
        let granted =
            EffectRow::from_effects(["Console".to_string(), "Clock".to_string(), "FS".to_string()]);
        if !row.row_vars().is_empty() || !row.concrete_effects().is_subset_of(&granted) {
            return Err(ProgramAdmissionError::UnsupportedEffectRow);
        }
    }

    let process_input = get("ProcessInput")?;
    let process_input_constructor = get("MkProcessInput")?;
    let program_caps = get("ProgramCaps")?;
    let program_caps_constructor = get("MkProgramCaps")?;
    let cap = get("Cap")?;
    let authority_constructor = get(authority_name)?;
    let host_io = get("HostIO")?;
    let host_exit = get("host_exit")?;
    let exit_code = get("ExitCode")?;
    let success_constructor = get("Success")?;
    let failure_constructor = get("Failure")?;
    let ret_constructor = get("Ret")?;
    let list_nil_constructor = get("Nil")?;
    let list_cons_constructor = get("Cons")?;
    let prod_constructor = get("MkProd")?;

    let actual = match env.env.lookup(main) {
        Some(Decl::Transparent { ty, .. })
        | Some(Decl::Opaque { ty, .. })
        | Some(Decl::Primitive { ty, .. }) => ty,
        _ => {
            return Err(ProgramAdmissionError::InvalidMainAbi {
                authority: authority_name.to_string(),
            })
        }
    };
    let authority_term = Term::constructor(authority_constructor, vec![]);
    let expected = Term::pi(
        Term::indformer(process_input, vec![]),
        Term::pi(
            Term::app(
                Term::indformer(program_caps, vec![]),
                authority_term.clone(),
            ),
            Term::app(
                Term::app(Term::const_(host_io, vec![]), authority_term),
                Term::indformer(exit_code, vec![]),
            ),
        ),
    );
    if !ken_kernel::convert_type(&env.env, &Context::new(), actual, &expected) {
        return Err(ProgramAdmissionError::InvalidMainAbi {
            authority: authority_name.to_string(),
        });
    }

    Ok(CheckedMainDescriptor {
        main,
        process_input,
        process_input_constructor,
        program_caps,
        program_caps_constructor,
        cap,
        authority_constructor,
        host_io,
        host_exit,
        exit_code,
        success_constructor,
        failure_constructor,
        ret_constructor,
        list_nil_constructor,
        list_cons_constructor,
        prod_constructor,
        authority_name: authority_name.to_string(),
        authority,
        allow_root_execution: header.allow_root_execution,
    })
}
