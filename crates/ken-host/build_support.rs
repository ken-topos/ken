use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Eq)]
struct ProducerInventory {
    o_flags: BTreeSet<String>,
    at_flags: BTreeSet<String>,
    modes: BTreeSet<String>,
    rustix_fs: BTreeSet<String>,
    std_fs: BTreeSet<String>,
    errno_kinds: BTreeSet<String>,
}

pub(crate) fn verify_inventory_closure(
    host_source: &str,
    consumer_source: &str,
    probe_source: &str,
    facts: &[(&str, u64)],
) -> Result<(), String> {
    let producer = derive_producer_inventory(host_source, consumer_source)?;
    let registry = registry_inventory(facts)?;

    compare_category("OFlags", &producer.o_flags, &registry.o_flags)?;
    compare_category("AtFlags", &producer.at_flags, &registry.at_flags)?;
    compare_category("Mode", &producer.modes, &registry.modes)?;
    compare_category("rustix fs", &producer.rustix_fs, &registry.rustix_fs)?;
    compare_category("std fs", &producer.std_fs, &registry.std_fs)?;
    compare_category("errno", &producer.errno_kinds, &registry.errno_kinds)?;

    let registered = facts
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let observed = extract_probe_labels(probe_source)?;
    for name in observed.difference(&registered) {
        return Err(format!("unregistered system-header observer fact: {name}"));
    }
    for name in registered.difference(&observed) {
        return Err(format!("manifested ABI fact lacks observer query: {name}"));
    }
    Ok(())
}

fn derive_producer_inventory(
    host_source: &str,
    consumer_source: &str,
) -> Result<ProducerInventory, String> {
    let production = host_source
        .split_once("#[cfg(test)]")
        .map(|(source, _)| source)
        .unwrap_or(host_source);
    let posix_consumer = consumer_source
        .split_once("impl HostHandler for PosixHost")
        .and_then(|(_, rest)| rest.split_once("/// Deterministic in-memory Console provider"))
        .map(|(source, _)| source)
        .ok_or_else(|| "cannot isolate the PosixHost consumer".to_owned())?;

    Ok(ProducerInventory {
        o_flags: identifiers_after(production, "OFlags::", None),
        at_flags: identifiers_after(production, "AtFlags::", None),
        modes: call_arguments(production, "Mode::from_raw_mode(")?,
        rustix_fs: call_identifiers_after(production, "fs::", Some("std::")),
        std_fs: call_identifiers_after(production, "std::fs::", None),
        errno_kinds: compared_error_kinds(posix_consumer),
    })
}

fn registry_inventory(facts: &[(&str, u64)]) -> Result<ProducerInventory, String> {
    let mut inventory = ProducerInventory {
        o_flags: BTreeSet::new(),
        at_flags: BTreeSet::from(["empty".to_owned()]),
        modes: BTreeSet::new(),
        rustix_fs: BTreeSet::new(),
        std_fs: BTreeSet::from(["read_dir".to_owned(), "remove_dir_all".to_owned()]),
        errno_kinds: BTreeSet::new(),
    };
    for (name, _) in facts {
        if let Some(flag) = name.strip_prefix("O_") {
            inventory.o_flags.insert(match flag {
                "CREAT" => "CREATE".to_owned(),
                other => other.to_owned(),
            });
        } else if let Some(flag) = name.strip_prefix("AT_") {
            inventory.at_flags.insert(flag.to_owned());
        } else if *name == "MODE_FILE_CREATE" {
            inventory.modes.insert("0o666".to_owned());
        } else if *name == "MODE_DIRECTORY_CREATE" {
            inventory.modes.insert("0o777".to_owned());
        } else if let Some(operation) = name.strip_prefix("SYS_") {
            inventory.rustix_fs.insert(operation.to_ascii_lowercase());
        } else if *name == "ERRNO_ENOENT" {
            inventory.errno_kinds.insert("NotFound".to_owned());
        } else if *name == "ERRNO_EEXIST" {
            inventory.errno_kinds.insert("AlreadyExists".to_owned());
        } else {
            return Err(format!("unclassified manifest ABI fact: {name}"));
        }
    }
    Ok(inventory)
}

fn compare_category(
    category: &str,
    producer: &BTreeSet<String>,
    registry: &BTreeSet<String>,
) -> Result<(), String> {
    if let Some(member) = producer.difference(registry).next() {
        return Err(format!(
            "unmanifested producer ABI fact: {category}::{member}"
        ));
    }
    if let Some(member) = registry.difference(producer).next() {
        return Err(format!(
            "manifested ABI fact lacks producer: {category}::{member}"
        ));
    }
    Ok(())
}

fn identifiers_after(
    source: &str,
    prefix: &str,
    excluded_before: Option<&str>,
) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(prefix) {
        let start = offset + relative;
        offset = start + prefix.len();
        if excluded_before.is_some_and(|excluded| source[..start].ends_with(excluded)) {
            continue;
        }
        let length = source[offset..]
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        if length > 0 {
            found.insert(source[offset..offset + length].to_owned());
        }
    }
    found
}

fn call_identifiers_after(
    source: &str,
    prefix: &str,
    excluded_before: Option<&str>,
) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(prefix) {
        let start = offset + relative;
        offset = start + prefix.len();
        if excluded_before.is_some_and(|excluded| source[..start].ends_with(excluded)) {
            continue;
        }
        let length = source[offset..]
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        if length == 0 {
            continue;
        }
        let after = source[offset + length..].trim_start();
        if after.starts_with('(') {
            found.insert(source[offset..offset + length].to_owned());
        }
    }
    found
}

fn call_arguments(source: &str, prefix: &str) -> Result<BTreeSet<String>, String> {
    let mut found = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(prefix) {
        let start = offset + relative + prefix.len();
        let mut depth = 1usize;
        let mut end = None;
        for (index, byte) in source[start..].bytes().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.ok_or_else(|| format!("unterminated producer call {prefix}"))?;
        let normalized = source[start..end]
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        found.insert(normalized);
        offset = end + 1;
    }
    Ok(found)
}

fn compared_error_kinds(source: &str) -> BTreeSet<String> {
    let prefix = "io::ErrorKind::";
    let mut found = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(".kind()") {
        let mut cursor = offset + relative + ".kind()".len();
        cursor += source[cursor..]
            .bytes()
            .take_while(u8::is_ascii_whitespace)
            .count();
        if !source[cursor..].starts_with("==") {
            offset = cursor;
            continue;
        }
        cursor += 2;
        cursor += source[cursor..]
            .bytes()
            .take_while(u8::is_ascii_whitespace)
            .count();
        if !source[cursor..].starts_with(prefix) {
            offset = cursor;
            continue;
        }
        cursor += prefix.len();
        let length = source[cursor..]
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        if length > 0 {
            found.insert(source[cursor..cursor + length].to_owned());
        }
        offset = cursor + length;
    }
    found
}

fn extract_probe_labels(source: &str) -> Result<BTreeSet<String>, String> {
    let prefix = "printf(\"";
    let mut labels = BTreeSet::new();
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(prefix) {
        let start = offset + relative + prefix.len();
        let rest = &source[start..];
        let end = rest
            .find("=%lld\\n\"")
            .ok_or_else(|| "observer printf must use fixed FACT=INTEGER protocol".to_owned())?;
        let label = &rest[..end];
        if label.is_empty()
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(format!("invalid observer fact label {label:?}"));
        }
        if !labels.insert(label.to_owned()) {
            return Err(format!("duplicate observer fact label {label}"));
        }
        offset = start + end + 1;
    }
    Ok(labels)
}

pub(crate) fn parse_probe(output: &str) -> Result<BTreeMap<String, u64>, String> {
    let mut facts = BTreeMap::new();
    for line in output.lines() {
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid probe line {line:?}"))?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(format!("invalid probe fact name {name:?}"));
        }
        let value = value
            .parse::<u64>()
            .map_err(|_| format!("invalid numeric probe value in {line:?}"))?;
        if facts.insert(name.to_owned(), value).is_some() {
            return Err(format!("duplicate probe fact {name}"));
        }
    }
    Ok(facts)
}

pub(crate) fn verify_probe(
    expected: &[(&str, u64)],
    observed: &BTreeMap<String, u64>,
) -> Result<(), String> {
    if observed.len() != expected.len() {
        return Err(format!(
            "probe emitted {} facts; expected the closed inventory of {}",
            observed.len(),
            expected.len()
        ));
    }
    for (name, expected_value) in expected {
        let observed_value = observed
            .get(*name)
            .ok_or_else(|| format!("probe omitted {name}"))?;
        if observed_value != expected_value {
            return Err(format!(
                "target ABI mismatch for {name}: linux-raw-sys={expected_value}, headers={observed_value}"
            ));
        }
    }
    Ok(())
}
