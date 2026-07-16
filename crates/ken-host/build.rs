mod build_support;

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SCHEMA_VERSION: u32 = 1;

fn main() {
    println!("cargo:rerun-if-changed=abi_probe.c");
    println!("cargo:rerun-if-changed=effect_abi_probe.c");
    println!("cargo:rerun-if-changed=effect_abi_v1.catalog");
    println!("cargo:rerun-if-changed=src/abi_v1/sigpipe.c");
    println!("cargo:rerun-if-changed=build_support.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=../ken-interp/src/eval.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=KEN_HOST_ABI_TEST_MISMATCH");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");

    let target = env::var("TARGET").expect("Cargo provides TARGET");
    let host = env::var("HOST").expect("Cargo provides HOST");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo provides target OS");
    let encoded_rustflags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    assert!(
        !encoded_rustflags.contains("rustix_use_libc")
            && !rustflags.contains("rustix_use_libc")
            && env::var_os("CARGO_CFG_MIRI").is_none(),
        "PX2 requires rustix's linux_raw backend; libc and Miri backends fail closed"
    );
    let manifest = fs::read_to_string("Cargo.toml").expect("read ken-host Cargo.toml");
    assert!(
        manifest.contains(
            "rustix = { version = \"=1.1.4\", default-features = false, features = [\"std\", \"fs\"] }"
        ),
        "PX2 manifest identity requires the exact audited rustix pin and features"
    );
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .and_then(Path::parent)
        .expect("ken-host is in crates/")
        .to_path_buf();
    let lock = fs::read_to_string(workspace.join("Cargo.lock")).expect("read workspace Cargo.lock");
    let dependencies = [
        package_identity(&lock, "rustix", "1.1.4", "std,fs"),
        package_identity(&lock, "bitflags", "2.13.0", ""),
        package_identity(&lock, "linux-raw-sys", "0.12.1", "std,general,errno"),
    ];

    if target_os == "linux" {
        compile_abi_v1_companion(&target, &host);
    }

    let (backend, facts): (&str, Vec<(&str, u64)>) = if target_os == "linux" && target == host {
        let facts = linux_raw_facts();
        verify_boundary_inventory(&facts);
        run_probe(&target, &host, &facts);
        ("linux_raw", facts)
    } else if target_os == "linux" {
        ("unavailable-cross-target", Vec::new())
    } else {
        ("unavailable-non-linux", Vec::new())
    };

    let effect_layout = if target_os == "linux" && target == host {
        run_effect_abi_probe(&target, &host)
    } else {
        Vec::new()
    };
    let effect_catalog = parse_effect_catalog();
    write_host_effect_generated(&target, &effect_catalog, &effect_layout);

    let canonical = canonical_manifest(&target, &target_os, backend, &dependencies, &facts);
    let hash: [u8; 32] = Sha256::digest(canonical.as_bytes()).into();
    write_generated(
        &target,
        &target_os,
        backend,
        &dependencies,
        &facts,
        &canonical,
        &hash,
    );
}

#[derive(Clone, Debug)]
struct EffectOp {
    name: String,
    id: u16,
    availability: String,
    request: String,
    request_arity: u8,
    reply: String,
    reply_arity: u8,
}

#[derive(Clone, Debug)]
struct EffectCatalog {
    operations: Vec<EffectOp>,
    bindings: Vec<(String, String, u64)>,
}

fn parse_effect_catalog() -> EffectCatalog {
    let source = fs::read_to_string("effect_abi_v1.catalog").expect("read effect ABI catalog");
    let mut operations = Vec::new();
    let mut bindings = Vec::new();
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
        let fields = line.split('|').collect::<Vec<_>>();
        match fields[0] {
            "operation" => {
                assert_eq!(fields.len(), 8, "effect operation rows have eight fields");
                operations.push(EffectOp {
                    name: fields[1].to_string(),
                    id: u16::from_str_radix(fields[2], 16).expect("effect op id is hex"),
                    availability: fields[3].to_string(),
                    request: fields[4].to_string(),
                    request_arity: fields[5].parse().expect("request arity is u8"),
                    reply: fields[6].to_string(),
                    reply_arity: fields[7].parse().expect("reply arity is u8"),
                });
                let operation = operations.last().unwrap();
                assert!(
                    matches!(operation.availability.as_str(), "native" | "unavailable"),
                    "effect availability is closed"
                );
                assert!(
                    operation.request.ends_with("RequestV1")
                        && operation.reply.ends_with("ReplyV1"),
                    "effect wire records are named V1 records"
                );
            }
            "schema" => {
                assert_eq!(fields.len(), 2, "effect schema row has two fields");
                bindings.push((
                    fields[0].to_string(),
                    "version".to_string(),
                    fields[1].parse().expect("effect schema is u64"),
                ));
            }
            "lifetime" | "tag" | "error" => {
                assert_eq!(fields.len(), 3, "effect binding rows have three fields");
                bindings.push((
                    fields[0].to_string(),
                    fields[1].to_string(),
                    fields[2].parse().expect("effect binding is u64"),
                ));
            }
            kind => panic!("unknown effect catalog row {kind}"),
        }
    }
    assert_eq!(operations.len(), 14, "HostOpV1 catalog is closed at 14");
    let mut ids = operations
        .iter()
        .map(|operation| operation.id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), operations.len(), "effect op ids are unique");
    let mut names = operations
        .iter()
        .map(|operation| operation.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), operations.len(), "effect op names are unique");
    let mut binding_keys = bindings
        .iter()
        .map(|(kind, name, _)| (kind, name))
        .collect::<Vec<_>>();
    binding_keys.sort_unstable();
    binding_keys.dedup();
    assert_eq!(
        binding_keys.len(),
        bindings.len(),
        "effect bindings are unique"
    );
    EffectCatalog {
        operations,
        bindings,
    }
}

fn run_effect_abi_probe(target: &str, host: &str) -> Vec<(String, u64)> {
    assert_eq!(target, host, "effect ABI headers attest only their target");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let executable = out_dir.join("ken-host-effect-abi-probe");
    let compiler = cc::Build::new().target(target).host(host).get_compiler();
    let mut compile = compiler.to_command();
    compile
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-Werror")
        .arg("effect_abi_probe.c")
        .arg("-o")
        .arg(&executable);
    assert!(compile
        .status()
        .expect("compile effect ABI probe")
        .success());
    let output = Command::new(executable)
        .output()
        .expect("run effect ABI probe");
    assert!(output.status.success(), "effect ABI probe failed closed");
    let stdout = String::from_utf8(output.stdout).expect("effect probe protocol is ASCII");
    let mut facts = stdout
        .lines()
        .map(|line| {
            let (name, value) = line
                .split_once('=')
                .expect("effect ABI probe emits NAME=INTEGER");
            assert!(
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
                "effect ABI fact name is closed ASCII"
            );
            (
                name.to_string(),
                value.parse::<u64>().expect("effect ABI fact is u64"),
            )
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| left.0.cmp(&right.0));
    facts
}

fn write_host_effect_generated(target: &str, catalog: &EffectCatalog, facts: &[(String, u64)]) {
    let mut canonical = format!("target={target}\n");
    for (kind, name, value) in &catalog.bindings {
        canonical.push_str(&format!("{kind}={name}|{value}\n"));
    }
    for operation in &catalog.operations {
        canonical.push_str(&format!(
            "operation={}|{:04x}|{}|{}|{}|{}|{}\n",
            operation.name,
            operation.id,
            operation.availability,
            operation.request,
            operation.request_arity,
            operation.reply,
            operation.reply_arity
        ));
    }
    for (name, value) in facts {
        canonical.push_str(&format!("layout={name}|{value}\n"));
    }
    let hash: [u8; 32] = Sha256::digest(canonical.as_bytes()).into();
    let fact_source = facts
        .iter()
        .map(|(name, value)| format!("({name:?}, {value}),"))
        .collect::<String>();
    let catalog_source = catalog
        .operations
        .iter()
        .map(|operation| {
            format!(
                "({:?}, {}, {:?}, {:?}, {}, {:?}, {}),",
                operation.name,
                operation.id,
                operation.availability,
                operation.request,
                operation.request_arity,
                operation.reply,
                operation.reply_arity
            )
        })
        .collect::<String>();
    let binding_source = catalog
        .bindings
        .iter()
        .map(|(kind, name, value)| format!("({kind:?}, {name:?}, {value}),"))
        .collect::<String>();
    let generated = format!(
        "pub const HOST_EFFECT_ABI_V1_CANONICAL: &str = {canonical:?};\n\
         pub const HOST_EFFECT_ABI_V1_HASH: [u8; 32] = {hash:?};\n\
         pub const HOST_EFFECT_ABI_V1_FACTS: &[(&str, u64)] = &[{fact_source}];\n\
         pub const HOST_EFFECT_ABI_V1_CATALOG: &[(&str, u16, &str, &str, u8, &str, u8)] = &[{catalog_source}];\n\
         pub const HOST_EFFECT_ABI_V1_BINDINGS: &[(&str, &str, u64)] = &[{binding_source}];\n"
    );
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("host_effect_abi_v1.rs"),
        generated,
    )
    .expect("write generated host effect ABI");
}

fn compile_abi_v1_companion(target: &str, host: &str) {
    cc::Build::new()
        .target(target)
        .host(host)
        .file("src/abi_v1/sigpipe.c")
        .warnings(true)
        .warnings_into_errors(true)
        .compile("ken_host_abi_v1_posture");
}

fn verify_boundary_inventory(facts: &[(&str, u64)]) {
    let build = fs::read_to_string("build.rs").expect("read landed ABI fact producer");
    let source = fs::read_to_string("src/lib.rs").expect("read landed ken-host producer");
    let consumer = fs::read_to_string("../ken-interp/src/eval.rs")
        .expect("read landed interpreter host-boundary consumer");
    let probe = fs::read_to_string("abi_probe.c").expect("read target ABI observer");
    build_support::verify_inventory_closure(&build, &source, &consumer, &probe, facts)
        .expect("producer, manifest, and observer ABI inventories must be identical");
}

fn package_identity(
    lock: &str,
    name: &str,
    version: &str,
    features: &str,
) -> (String, String, String, String) {
    for section in lock.split("[[package]]").skip(1) {
        let field = |key: &str| {
            section.lines().find_map(|line| {
                line.strip_prefix(&format!("{key} = \""))
                    .and_then(|value| value.strip_suffix('"'))
            })
        };
        if field("name") == Some(name) && field("version") == Some(version) {
            return (
                name.to_owned(),
                version.to_owned(),
                field("checksum")
                    .expect("registry dependency has checksum")
                    .to_owned(),
                features.to_owned(),
            );
        }
    }
    panic!("Cargo.lock lacks exact {name} {version}");
}

#[cfg(target_os = "linux")]
fn linux_raw_facts() -> Vec<(&'static str, u64)> {
    use linux_raw_sys::{errno, general};
    vec![
        width_fact("POINTER_WIDTH", bit_width::<usize>()),
        width_fact("C_INT_WIDTH", bit_width::<core::ffi::c_int>()),
        ("O_RDONLY", general::O_RDONLY.into()),
        ("O_WRONLY", general::O_WRONLY.into()),
        ("O_RDWR", general::O_RDWR.into()),
        ("O_APPEND", general::O_APPEND.into()),
        ("O_CREAT", general::O_CREAT.into()),
        ("O_EXCL", general::O_EXCL.into()),
        ("O_TRUNC", general::O_TRUNC.into()),
        ("O_DIRECTORY", general::O_DIRECTORY.into()),
        ("O_NOFOLLOW", general::O_NOFOLLOW.into()),
        ("O_CLOEXEC", general::O_CLOEXEC.into()),
        ("AT_REMOVEDIR", general::AT_REMOVEDIR.into()),
        (
            "MODE_FILE_CREATE",
            (general::S_IRUSR
                | general::S_IWUSR
                | general::S_IRGRP
                | general::S_IWGRP
                | general::S_IROTH
                | general::S_IWOTH)
                .into(),
        ),
        (
            "MODE_DIRECTORY_CREATE",
            (general::S_IRWXU | general::S_IRWXG | general::S_IRWXO).into(),
        ),
        ("SYS_OPENAT", general::__NR_openat.into()),
        ("SYS_MKDIRAT", general::__NR_mkdirat.into()),
        ("SYS_UNLINKAT", general::__NR_unlinkat.into()),
        ("SYS_RENAMEAT", general::__NR_renameat.into()),
        ("SYS_READLINKAT", general::__NR_readlinkat.into()),
        ("ERRNO_ENOENT", errno::ENOENT.into()),
        ("ERRNO_EEXIST", errno::EEXIST.into()),
    ]
}

#[cfg(target_os = "linux")]
fn bit_width<T>() -> u64 {
    (core::mem::size_of::<T>() * u8::BITS as usize) as u64
}

#[cfg(target_os = "linux")]
fn width_fact(name: &'static str, value: u64) -> (&'static str, u64) {
    (name, value)
}

#[cfg(not(target_os = "linux"))]
fn linux_raw_facts() -> Vec<(&'static str, u64)> {
    Vec::new()
}

fn run_probe(target: &str, host: &str, expected: &[(&str, u64)]) {
    assert_eq!(
        target, host,
        "system headers may only attest their own target"
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let executable = out_dir.join("ken-host-abi-probe");
    let compiler = cc::Build::new().target(target).host(host).get_compiler();
    let mut compile = compiler.to_command();
    compile.arg("abi_probe.c").arg("-o").arg(&executable);
    let status = compile.status().expect("run target-qualified C compiler");
    assert!(
        status.success(),
        "target ABI probe compilation failed closed"
    );
    let output = Command::new(&executable)
        .output()
        .expect("run target ABI probe for the manifested target");
    assert!(
        output.status.success(),
        "target ABI probe execution failed closed"
    );
    let stdout = String::from_utf8(output.stdout).expect("probe protocol is ASCII");
    let observed = build_support::parse_probe(&stdout).expect("parse closed FACT=INTEGER protocol");
    let mut checked = expected.to_vec();
    if env::var_os("KEN_HOST_ABI_TEST_MISMATCH").is_some() {
        checked[0].1 ^= 1;
    }
    build_support::verify_probe(&checked, &observed)
        .expect("system headers disagree with linux-raw-sys");
}

fn canonical_manifest(
    target: &str,
    target_os: &str,
    backend: &str,
    dependencies: &[(String, String, String, String); 3],
    facts: &[(&str, u64)],
) -> String {
    let mut out = format!(
        "schema={SCHEMA_VERSION}\ntarget={target}\ntarget_os={target_os}\nbackend={backend}\n"
    );
    for (name, version, checksum, features) in dependencies {
        out.push_str(&format!(
            "dependency={name}|{version}|{checksum}|{features}\n"
        ));
    }
    out.push_str(&format!("fact_count={}\n", facts.len()));
    for (name, value) in facts {
        out.push_str(&format!("fact={name}|{value}\n"));
    }
    out
}

fn write_generated(
    target: &str,
    target_os: &str,
    backend: &str,
    dependencies: &[(String, String, String, String); 3],
    facts: &[(&str, u64)],
    canonical: &str,
    hash: &[u8; 32],
) {
    let dependencies = dependencies.iter().map(|(name, version, checksum, features)| {
        format!("DependencyIdentity {{ name: {name:?}, version: {version:?}, checksum: {checksum:?}, features: &{:?} }},", features.split(',').filter(|feature| !feature.is_empty()).collect::<Vec<_>>())
    }).collect::<String>();
    let facts = facts
        .iter()
        .map(|(name, value)| format!("AbiFact {{ name: {name:?}, value: {value} }},"))
        .collect::<String>();
    let generated = format!(
        "pub const TARGET_ABI_CANONICAL: &str = {canonical:?};\n\
         pub const TARGET_ABI_MANIFEST_HASH: [u8; 32] = {hash:?};\n\
         pub const TARGET_ABI: TargetAbi = TargetAbi {{ schema_version: {SCHEMA_VERSION}, target: {target:?}, target_os: {target_os:?}, backend: {backend:?}, dependencies: &[{dependencies}], fact_count: {fact_count}, facts: &[{facts}], manifest_hash: TARGET_ABI_MANIFEST_HASH }};\n",
        fact_count = facts.matches("AbiFact").count(),
    );
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("target_abi.rs"),
        generated,
    )
    .expect("write generated TargetAbi module");
}
