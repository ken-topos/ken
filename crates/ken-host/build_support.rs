use std::collections::BTreeMap;

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
