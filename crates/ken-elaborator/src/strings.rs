//! Normalized semantic string carrier (`37 §2.1`).

use std::fmt;
use std::ops::Deref;

use unicode_normalization::UnicodeNormalization;

/// A UTF-8 string normalized to NFC at construction.
///
/// The raw buffer is private so semantic string producers cannot bypass the
/// normalization boundary.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NfcString(String);

impl NfcString {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().nfc().collect())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for NfcString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for NfcString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for NfcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for NfcString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NfcString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for NfcString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NfcString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for NfcString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}
