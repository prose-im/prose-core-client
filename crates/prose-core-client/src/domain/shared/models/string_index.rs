// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Range;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Utf8Index(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Utf16Index(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnicodeScalarIndex(usize);

impl Utf8Index {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn to_scalar_index(&self, string: &str) -> Result<UnicodeScalarIndex> {
        let mut utf8_idx = self.0;
        let mut scalar_idx = 0;

        for c in string.chars() {
            if utf8_idx == 0 {
                break;
            }

            utf8_idx = utf8_idx
                .checked_sub(c.len_utf8())
                .ok_or(anyhow!("Utf8Index is not at a char boundary."))?;

            scalar_idx += 1;
        }

        (utf8_idx == 0)
            .then_some(UnicodeScalarIndex(scalar_idx))
            .ok_or(anyhow!("Utf8Index is out of bounds."))
    }
}

impl Utf16Index {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn to_scalar_index(&self, string: &str) -> Result<UnicodeScalarIndex> {
        let mut utf16_idx = self.0;
        let mut scalar_idx = 0;

        for c in string.chars() {
            if utf16_idx == 0 {
                break;
            }

            utf16_idx = utf16_idx
                .checked_sub(c.len_utf16())
                .ok_or(anyhow!("Utf16Index is not at a char boundary."))?;

            scalar_idx += 1;
        }

        (utf16_idx == 0)
            .then_some(UnicodeScalarIndex(scalar_idx))
            .ok_or(anyhow!("Utf16Index is out of bounds."))
    }
}

impl UnicodeScalarIndex {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn to_utf16_index(&self, string: &str) -> Result<Utf16Index> {
        let iter = string.chars().take(self.0);

        if iter.count() < self.0 {
            bail!("UnicodeScalarIndex is out of bounds.")
        }

        Ok(Utf16Index(
            string.chars().take(self.0).map(|c| c.len_utf16()).sum(),
        ))
    }
}

impl AsRef<usize> for Utf16Index {
    fn as_ref(&self) -> &usize {
        &self.0
    }
}

impl AsRef<usize> for UnicodeScalarIndex {
    fn as_ref(&self) -> &usize {
        &self.0
    }
}

pub trait StringIndexRangeExt {
    fn to_scalar_range(&self, string: &str) -> Result<Range<UnicodeScalarIndex>>;
}

pub trait ScalarRangeExt {
    fn to_utf16_range(&self, string: &str) -> Result<Range<Utf16Index>>;
}

impl StringIndexRangeExt for Range<Utf8Index> {
    fn to_scalar_range(&self, string: &str) -> Result<Range<UnicodeScalarIndex>> {
        Ok(Range {
            start: self.start.to_scalar_index(string)?,
            end: self.end.to_scalar_index(string)?,
        })
    }
}

impl StringIndexRangeExt for Range<Utf16Index> {
    fn to_scalar_range(&self, string: &str) -> Result<Range<UnicodeScalarIndex>> {
        Ok(Range {
            start: self.start.to_scalar_index(string)?,
            end: self.end.to_scalar_index(string)?,
        })
    }
}

impl ScalarRangeExt for Range<UnicodeScalarIndex> {
    fn to_utf16_range(&self, string: &str) -> Result<Range<Utf16Index>> {
        Ok(Range {
            start: self.start.to_utf16_index(string)?,
            end: self.end.to_utf16_index(string)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_to_scalar_index() -> Result<()> {
        assert_eq!(
            UnicodeScalarIndex::new(0),
            Utf8Index::new(0).to_scalar_index("")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(6),
            Utf8Index::new(6).to_scalar_index("Hello, World!")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(1),
            Utf8Index::new(4).to_scalar_index("𐐀")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(1),
            Utf8Index::new(2).to_scalar_index("ñ")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(7),
            Utf8Index::new(25).to_scalar_index("👩‍👩‍👧‍👦")?,
        );
        assert!(Utf8Index::new(1).to_scalar_index("👩‍👩‍👧‍👦").is_err());
        assert!(Utf8Index::new(4).to_scalar_index("123").is_err());
        Ok(())
    }

    #[test]
    fn test_utf16_to_scalar_index() -> Result<()> {
        assert_eq!(
            UnicodeScalarIndex::new(0),
            Utf16Index::new(0).to_scalar_index("")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(6),
            Utf16Index::new(6).to_scalar_index("Hello, World!")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(1),
            Utf16Index::new(2).to_scalar_index("𐐀")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(1),
            Utf16Index::new(1).to_scalar_index("ñ")?,
        );
        assert_eq!(
            UnicodeScalarIndex::new(7),
            Utf16Index::new(11).to_scalar_index("👩‍👩‍👧‍👦")?,
        );
        assert!(Utf16Index::new(1).to_scalar_index("👩‍👩‍👧‍👦").is_err());
        assert!(Utf16Index::new(4).to_scalar_index("123").is_err());
        Ok(())
    }

    #[test]
    fn test_scalar_to_utf16_index() -> Result<()> {
        assert_eq!(
            Utf16Index::new(0),
            UnicodeScalarIndex::new(0).to_utf16_index("")?,
        );
        assert_eq!(
            Utf16Index::new(6),
            UnicodeScalarIndex::new(6).to_utf16_index("Hello, World!")?,
        );
        assert_eq!(
            Utf16Index::new(2),
            UnicodeScalarIndex::new(1).to_utf16_index("𐐀")?,
        );
        assert_eq!(
            Utf16Index::new(1),
            UnicodeScalarIndex::new(1).to_utf16_index("ñ")?,
        );
        assert_eq!(
            Utf16Index::new(11),
            UnicodeScalarIndex::new(7).to_utf16_index("👩‍👩‍👧‍👦")?,
        );
        assert!(UnicodeScalarIndex::new(4).to_utf16_index("123").is_err());
        Ok(())
    }
}
