//! NC6 Cranelift backend spike for the NC5 runtime IR seed.
//!
//! This module deliberately keeps the native boundary narrow. Cranelift code
//! returns scalar `i64` values directly and aggregate observations through an
//! opaque token table decoded by this Rust layer. Native addresses, object
//! layout, allocation order, ABI details, and Cranelift internals never become
//! Ken-observable meaning.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::mem;

use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, UserFuncName};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::{verify_function, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};

use crate::{
    RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExample, RuntimeExpr, RuntimeGroundValue,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimePartiality, RuntimePrimitive,
    RuntimeProgram, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftRunReport {
    pub example: String,
    pub observation: RuntimeObservation,
    pub verifier_passed: bool,
    pub native_returned: Option<i64>,
    pub trust: NativeTrustReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeTrustReport {
    pub backend: &'static str,
    pub fidelity: NativeFidelity,
    pub verifier_passed: bool,
    pub assumptions: BTreeSet<String>,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeFidelity {
    F0NativeExample,
    F1SeedObservationAgreement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CraneliftBackendError {
    Unsupported(UnsupportedLowering),
    Backend(BackendFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedLowering {
    pub construct: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendFailure {
    Target(String),
    Verifier(String),
    Module(String),
    NativeResultDecode { token: i64 },
}

impl fmt::Display for CraneliftBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CraneliftBackendError::Unsupported(err) => {
                write!(f, "unsupported runtime-IR lowering: {err}")
            }
            CraneliftBackendError::Backend(err) => write!(f, "Cranelift backend failure: {err}"),
        }
    }
}

impl std::error::Error for CraneliftBackendError {}

impl fmt::Display for UnsupportedLowering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.construct, self.reason)
    }
}

impl fmt::Display for BackendFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendFailure::Target(msg) => write!(f, "target setup failed: {msg}"),
            BackendFailure::Verifier(msg) => write!(f, "verifier rejected function: {msg}"),
            BackendFailure::Module(msg) => write!(f, "module operation failed: {msg}"),
            BackendFailure::NativeResultDecode { token } => {
                write!(f, "native result token {token} is not in the result table")
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeSeedEnvironment {
    values: BTreeMap<String, RuntimeGroundValue>,
}

impl NativeSeedEnvironment {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn nc5_seed() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "decl:fixture::Local::y".to_string(),
            RuntimeGroundValue::Int(2),
        );
        Self { values }
    }

    pub fn insert(&mut self, symbol: impl Into<String>, value: RuntimeGroundValue) {
        self.values.insert(symbol.into(), value);
    }
}

pub fn run_nc6_seed_examples(
    program: &RuntimeProgram,
) -> Result<Vec<CraneliftRunReport>, CraneliftBackendError> {
    reject_program_blockers(program)?;
    let env = NativeSeedEnvironment::nc5_seed();
    program
        .examples
        .iter()
        .map(|example| run_example_with_seed_observation(example, &env))
        .collect()
}

pub fn reject_program_blockers(program: &RuntimeProgram) -> Result<(), CraneliftBackendError> {
    for declaration in &program.declarations {
        if declaration.metadata.unsupported.is_some()
            || program
                .erased_core
                .metadata
                .unsupported
                .contains_key(&declaration.symbol)
        {
            return Err(unsupported(
                "RuntimeProgram",
                format!("reachable unsupported entry {}", declaration.symbol),
            ));
        }

        let lowerability = declaration
            .metadata
            .lowerability
            .as_ref()
            .or_else(|| {
                program
                    .erased_core
                    .metadata
                    .lowerability
                    .get(&declaration.symbol)
            })
            .ok_or_else(|| {
                unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} is missing runtime lowerability metadata",
                        declaration.symbol
                    ),
                )
            })?;
        if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} has blocking lowerability metadata: {:?}",
                    declaration.symbol, lowerability
                ),
            ));
        }

        if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
            if !effects.is_empty() {
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} declares effects outside the NC6 D1 supported subset",
                        declaration.symbol
                    ),
                ));
            }
        }

        if let Some(effect_meta) = program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .get(&declaration.symbol)
        {
            if effect_meta.boundary == RuntimeEffectBoundary::Foreign
                || effect_meta.foreign_symbol.is_some()
            {
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} crosses a foreign boundary outside the NC6 D1 subset",
                        declaration.symbol
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn run_example_with_seed_observation(
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = compile_expr(&example.ir, env)?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run()?;
    let fidelity = if observation == example.observation {
        NativeFidelity::F1SeedObservationAgreement
    } else {
        NativeFidelity::F0NativeExample
    };
    Ok(CraneliftRunReport {
        example: example.name.clone(),
        observation,
        verifier_passed,
        native_returned,
        trust: NativeTrustReport {
            backend: "Cranelift JIT",
            fidelity,
            verifier_passed,
            assumptions,
            unsupported,
        },
    })
}

struct CompiledExpr {
    module: JITModule,
    func_id: cranelift_module::FuncId,
    decoder: Option<ResultDecoder>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    trap: Option<RuntimeTrap>,
    verifier_passed: bool,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
}

#[derive(Clone, Copy)]
enum ResultDecoder {
    Int,
    Bool,
    Table,
}

impl CompiledExpr {
    fn run(mut self) -> Result<(RuntimeObservation, Option<i64>), CraneliftBackendError> {
        if let Some(trap) = self.trap {
            return Ok((RuntimeObservation::Trapped(trap), None));
        }

        self.module
            .finalize_definitions()
            .map_err(|err| backend_module(err.to_string()))?;
        let code = self.module.get_finalized_function(self.func_id);
        let native = unsafe { mem::transmute::<_, extern "C" fn() -> i64>(code) };
        let token = native();
        let decoder = self
            .decoder
            .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?;
        let ground = match decoder {
            ResultDecoder::Int => RuntimeGroundValue::Int(token),
            ResultDecoder::Bool => RuntimeGroundValue::Bool(token != 0),
            ResultDecoder::Table => self
                .result_table
                .get(&token)
                .cloned()
                .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
        };
        Ok((RuntimeObservation::Returned(ground), Some(token)))
    }
}

fn compile_expr(
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut sig = module.make_signature();
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function("ken_nc6_seed", Linkage::Local, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let mut ctx = Context::new();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        result_table: BTreeMap::new(),
        next_token: 0,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
    };
    let (maybe_trap, decoder) = {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        let lowered = compiler.lower_expr(&mut builder, expr, &[])?;
        let result = match lowered {
            Lowered::Trap(trap) => {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                (Some(trap), None)
            }
            value => {
                let (token, decoder) = compiler.emit_result(&mut builder, value)?;
                builder.ins().return_(&[token]);
                (None, Some(decoder))
            }
        };
        builder.seal_all_blocks();
        builder.finalize();
        result
    };

    verify_cranelift_function(&ctx.func, module.isa())?;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))?;

    Ok(CompiledExpr {
        module,
        func_id,
        decoder,
        result_table: compiler.result_table,
        trap: maybe_trap,
        verifier_passed: true,
        assumptions: compiler.assumptions,
        unsupported: compiler.unsupported,
    })
}

fn new_jit_module() -> Result<JITModule, CraneliftBackendError> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    Ok(JITModule::new(builder))
}

fn verify_cranelift_function(
    func: &Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(), CraneliftBackendError> {
    verify_function(func, isa).map_err(|err| backend(BackendFailure::Verifier(err.to_string())))
}

struct Lowering<'a> {
    seed_env: &'a NativeSeedEnvironment,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    next_token: i64,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
}

#[derive(Clone)]
enum Lowered {
    Int(cranelift_codegen::ir::Value),
    Bool(cranelift_codegen::ir::Value),
    Bytes(Vec<u8>),
    String(String),
    Constructor {
        constructor: String,
        args: Vec<Lowered>,
    },
    Record {
        fields: Vec<(String, Lowered)>,
    },
    Closure {
        captures: Vec<Lowered>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
    Trap(RuntimeTrap),
}

impl<'a> Lowering<'a> {
    fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        match expr {
            RuntimeExpr::Value(value) => self.lower_value(builder, value),
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| unsupported("Var", format!("no runtime binding for index {index}"))),
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.lower_primitive_call(builder, primitive, args, env)
            }
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Constructor {
                    constructor: constructor.clone(),
                    args: lowered_args,
                })
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                let Lowered::Constructor { constructor, args } = lowered_scrutinee else {
                    return Err(unsupported(
                        "Match",
                        "scrutinee is not a constructor value in the NC6 subset",
                    ));
                };
                let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
                    return Ok(Lowered::Trap(default.clone()));
                };
                if case.binders != args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let mut case_env = args;
                case_env.extend_from_slice(env);
                self.lower_expr(builder, &case.body, &case_env)
            }
            RuntimeExpr::Record { fields } => {
                let lowered_fields = fields
                    .iter()
                    .map(|(name, expr)| Ok((name.clone(), self.lower_expr(builder, expr, env)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                Ok(Lowered::Record {
                    fields: lowered_fields,
                })
            }
            RuntimeExpr::Project { record, field } => {
                let lowered_record = self.lower_expr(builder, record, env)?;
                let Lowered::Record { fields } = lowered_record else {
                    return Err(unsupported(
                        "Project",
                        "record projection needs a record value",
                    ));
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *field).then_some(value))
                    .ok_or_else(|| unsupported("Project", format!("missing field {field}")))
            }
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let lowered_captures = captures
                    .iter()
                    .map(|symbol| self.lower_seed_capture(builder, symbol))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Closure {
                    captures: lowered_captures,
                    params: params.clone(),
                    body: (**body).clone(),
                })
            }
            RuntimeExpr::Call { callee, args } => {
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                let Lowered::Closure {
                    captures,
                    params,
                    body,
                } = lowered_callee
                else {
                    return Err(unsupported("Call", "callee is not a closure"));
                };
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut call_env = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                call_env.extend(captures);
                call_env.extend_from_slice(env);
                self.lower_expr(builder, &body, &call_env)
            }
            RuntimeExpr::Trap(trap) => Ok(Lowered::Trap(trap.clone())),
            RuntimeExpr::Let { .. } => Err(unsupported(
                "Let",
                "let lowering is outside the NC6 seed-example subset",
            )),
            RuntimeExpr::If { .. } => Err(unsupported(
                "If",
                "branch lowering is outside the NC6 seed-example subset",
            )),
            RuntimeExpr::Effect { effect, .. } => Err(unsupported(
                "Effect",
                format!("effect {effect} is not modeled in the NC6 D1 subset"),
            )),
        }
    }

    fn lower_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeValue::Bool(value) => Ok(Lowered::Bool(
                builder.ins().iconst(types::I64, i64::from(*value)),
            )),
            RuntimeValue::Int(value) => Ok(Lowered::Int(builder.ins().iconst(types::I64, *value))),
            RuntimeValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.lower_value(builder, value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            RuntimeValue::ClosureRef { .. } => Err(unsupported(
                "ClosureRef",
                "pre-existing closure references are not lowered by the NC6 seed backend",
            )),
            RuntimeValue::Unknown => Err(unsupported(
                "Unknown",
                "unknown runtime values must reject before backend lowering",
            )),
        }
    }

    fn lower_seed_capture(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &str,
    ) -> Result<Lowered, CraneliftBackendError> {
        let value = self.seed_env.values.get(symbol).ok_or_else(|| {
            unsupported(
                "Closure",
                format!("capture {symbol} has no runtime value in the seed environment"),
            )
        })?;
        self.lower_ground_value(builder, value)
    }

    fn lower_ground_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeGroundValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeGroundValue::Bool(value) => Ok(Lowered::Bool(
                builder.ins().iconst(types::I64, i64::from(*value)),
            )),
            RuntimeGroundValue::Int(value) => {
                Ok(Lowered::Int(builder.ins().iconst(types::I64, *value)))
            }
            RuntimeGroundValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeGroundValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeGroundValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_ground_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeGroundValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Ok((name.clone(), self.lower_ground_value(builder, value)?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
        }
    }

    fn lower_primitive_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, env))
            .collect::<Result<Vec<_>, _>>()?;

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                self.assumptions.insert(format!(
                    "checked partial obligation {obligation} not discharged"
                ));
                let message = if obligation.ends_with(".bounds") {
                    format!("{} bounds obligation failed", primitive.symbol)
                } else {
                    format!("{} checked partiality trapped", primitive.symbol)
                };
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message,
                }));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: format!("{} trusted partiality trapped", primitive.symbol),
                }));
            }
        }

        match primitive.symbol.as_str() {
            "add_int" => {
                if args.len() != 2 {
                    return Err(unsupported(
                        "PrimitiveCall",
                        format!("add_int expects 2 args, got {}", args.len()),
                    ));
                }
                let mut lowered_args = lowered_args.into_iter();
                let lhs = lowered_args.next().expect("arg count checked");
                let rhs = lowered_args.next().expect("arg count checked");
                let (Lowered::Int(lhs), Lowered::Int(rhs)) = (lhs, rhs) else {
                    return Err(unsupported(
                        "PrimitiveCall",
                        "add_int only supports Int arguments in NC6 D1",
                    ));
                };
                Ok(Lowered::Int(builder.ins().iadd(lhs, rhs)))
            }
            other => Err(unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the NC6 D1 supported set"),
            )),
        }
    }

    fn emit_result(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, ResultDecoder), CraneliftBackendError> {
        match value {
            Lowered::Int(value) => Ok((value, ResultDecoder::Int)),
            Lowered::Bool(value) => Ok((value, ResultDecoder::Bool)),
            value => {
                let ground = self.ground_value(value)?;
                let token = self.intern_result(ground);
                Ok((
                    builder.ins().iconst(types::I64, token),
                    ResultDecoder::Table,
                ))
            }
        }
    }

    fn ground_value(
        &mut self,
        value: Lowered,
    ) -> Result<RuntimeGroundValue, CraneliftBackendError> {
        match value {
            Lowered::Int(_) => Err(unsupported(
                "Result",
                "internal error: Int scalar should decode directly from native return",
            )),
            Lowered::Bool(_) => Err(unsupported(
                "Result",
                "internal error: Bool scalar should decode directly from native return",
            )),
            Lowered::Bytes(value) => Ok(RuntimeGroundValue::Bytes(value)),
            Lowered::String(value) => Ok(RuntimeGroundValue::String(value)),
            Lowered::Constructor { constructor, args } => Ok(RuntimeGroundValue::Constructor {
                constructor,
                args: args
                    .into_iter()
                    .map(|arg| self.ground_value(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            Lowered::Record { fields } => Ok(RuntimeGroundValue::Record {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| Ok((name, self.ground_value(value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            Lowered::Closure { .. } => Err(unsupported(
                "Closure",
                "closures are callable but not observable ground values in NC6 D1",
            )),
            Lowered::Trap(trap) => Err(unsupported(
                "Trap",
                format!("trap result must be reported as trapped: {}", trap.message),
            )),
        }
    }

    fn intern_result(&mut self, ground: RuntimeGroundValue) -> i64 {
        let token = self.next_token;
        self.next_token += 1;
        self.result_table.insert(token, ground);
        token
    }
}

fn unsupported(construct: &'static str, reason: impl Into<String>) -> CraneliftBackendError {
    CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct,
        reason: reason.into(),
    })
}

fn backend(failure: BackendFailure) -> CraneliftBackendError {
    CraneliftBackendError::Backend(failure)
}

fn backend_module(reason: String) -> CraneliftBackendError {
    backend(BackendFailure::Module(reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        nc5_seed_examples, ErasedExecutableCore, RuntimeDeclaration, RuntimeFieldStatus,
        RuntimeMatchCase, RuntimeMetadata, RuntimeSymbolMetadata,
    };

    fn seed_program_with_lowerability(status: Option<RuntimeLowerabilityStatus>) -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        if let Some(status) = status.clone() {
            metadata.lowerability.insert(symbol.clone(), status);
        }
        RuntimeProgram {
            package_identity: "module:fixture::nc6".to_string(),
            core_semantic_hash: 1,
            artifact_hash: 2,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol,
                kind: RuntimeDeclarationKind::Record {
                    fields: vec![crate::RuntimeField {
                        name: "value".to_string(),
                        status: RuntimeFieldStatus::Runtime,
                    }],
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: status,
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: nc5_seed_examples(),
        }
    }

    #[test]
    fn cranelift_runs_scalar_seed_and_verifies_function() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists");

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
        assert_eq!(
            report.trust.fidelity,
            NativeFidelity::F1SeedObservationAgreement
        );
    }

    #[test]
    fn cranelift_runs_constructor_match_and_record_projection_seeds() {
        let env = NativeSeedEnvironment::empty();
        for name in ["adt-constructor-match", "record-construction-projection"] {
            let example = nc5_seed_examples()
                .into_iter()
                .find(|example| example.name == name)
                .expect("seed exists");

            let report =
                run_example_with_seed_observation(&example, &env).expect("native run succeeds");

            assert!(report.verifier_passed);
            assert_eq!(report.observation, example.observation);
        }
    }

    #[test]
    fn cranelift_reports_bytes_and_string_immediates_as_ground_values() {
        for (name, ir, observation) in [
            (
                "bytes-immediate",
                RuntimeExpr::Value(RuntimeValue::Bytes(vec![1, 2, 3])),
                RuntimeObservation::Returned(RuntimeGroundValue::Bytes(vec![1, 2, 3])),
            ),
            (
                "string-immediate",
                RuntimeExpr::Value(RuntimeValue::String("ken".to_string())),
                RuntimeObservation::Returned(RuntimeGroundValue::String("ken".to_string())),
            ),
        ] {
            let example = RuntimeExample {
                name: name.to_string(),
                checked_core_shape: "diagnostic label only".to_string(),
                ir,
                observation,
            };

            let report =
                run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
                    .expect("native run succeeds");

            assert!(report.verifier_passed);
            assert_eq!(report.observation, example.observation);
            assert!(
                report.native_returned.is_some(),
                "native function returns an opaque table token"
            );
        }
    }

    #[test]
    fn cranelift_runs_closure_seed_with_explicit_runtime_capture_environment() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closure-capture-application")
            .expect("seed exists");

        let report =
            run_example_with_seed_observation(&example, &NativeSeedEnvironment::nc5_seed())
                .expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("missing capture must reject loudly");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Closure",
                ..
            })
        ));
    }

    #[test]
    fn program_runner_preflights_metadata_before_backend_lowering() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));

        let reports = run_nc6_seed_examples(&program).expect("seed program runs");

        assert_eq!(reports.len(), 5);
        assert!(reports
            .iter()
            .all(|report| report.trust.fidelity == NativeFidelity::F1SeedObservationAgreement));
    }

    #[test]
    fn missing_lowerability_metadata_rejects_before_backend_lowering() {
        let program = seed_program_with_lowerability(None);

        let err = run_nc6_seed_examples(&program).expect_err("missing metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_unsupported_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .unsupported
            .insert(symbol, b"unsupported target".to_vec());

        let err = run_nc6_seed_examples(&program).expect_err("unsupported metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn explicit_partial_primitive_reports_trap_not_backend_bug() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "explicit-partial-primitive-trap")
            .expect("seed exists");

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("trap report succeeds");

        assert!(report.verifier_passed);
        assert!(matches!(
            report.observation,
            RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                ..
            })
        ));
    }

    #[test]
    fn checked_partial_primitive_still_rejects_unknown_arguments() {
        let example = RuntimeExample {
            name: "unknown-partial-arg".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_at".to_string(),
                    partiality: RuntimePartiality::CheckedTrap {
                        obligation: "obl:bytes_at.bounds".to_string(),
                    },
                },
                args: vec![RuntimeExpr::Value(RuntimeValue::Unknown)],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "unused".to_string(),
            }),
        };

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("unknown argument must reject before trap reporting");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Unknown",
                ..
            })
        ));
    }

    #[test]
    fn unsupported_effect_is_distinct_from_backend_failure() {
        let example = RuntimeExample {
            name: "unsupported-effect".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Effect {
                effect: "Console".to_string(),
                capability: Some("cap:Console".to_string()),
                args: vec![],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::UnsupportedErasure,
                message: "unsupported".to_string(),
            }),
        };

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("effect must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Effect",
                ..
            })
        ));
    }

    #[test]
    fn pattern_default_trap_is_observation_not_backend_error() {
        let example = RuntimeExample {
            name: "match-default".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:None".to_string(),
                    args: vec![],
                }),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:Some".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "no case selected".to_string(),
                },
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "no case selected".to_string(),
            }),
        };

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("trap report succeeds");

        assert_eq!(report.observation, example.observation);
    }
}
