//! CAT-TAX — controlled top-level catalog Section allowlist.

use std::fs;
use std::path::Path;

const ALLOWED_SECTIONS: [&str; 7] = [
    "Core",
    "Data",
    "Algorithm",
    "Capability",
    "Protocol",
    "Application",
    "Tooling",
];

fn validate_sections<'a>(sections: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    for section in sections {
        if !ALLOWED_SECTIONS.contains(&section) {
            return Err(format!(
                "catalog package root `{section}` is not an allowed Section"
            ));
        }
    }
    Ok(())
}

#[test]
fn catalog_package_roots_use_only_the_controlled_sections() {
    let packages = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("catalog/packages");
    let mut sections = fs::read_dir(packages)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            entry
                .file_type()
                .unwrap()
                .is_dir()
                .then(|| entry.file_name())
        })
        .map(|name| name.into_string().unwrap())
        .collect::<Vec<_>>();
    sections.sort();

    validate_sections(sections.iter().map(String::as_str)).unwrap();
    assert_eq!(
        sections,
        ["Application", "Capability", "Core", "Data", "Tooling"],
        "reserved Sections stay absent until their first package lands"
    );
}

#[test]
fn catalog_package_root_lint_rejects_an_unlisted_probe() {
    let error = validate_sections(["Core", "Misc"]).unwrap_err();
    assert!(error.contains("Misc"), "{error}");
}
