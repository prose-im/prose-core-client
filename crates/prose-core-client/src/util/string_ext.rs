// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::Bound;
use std::ops::RangeBounds;

pub trait StringExt {
    fn to_uppercase_first_letter(&self) -> String;

    /// Converts the username part of a JID into a human-readable capitalized display name.
    ///
    /// This method takes the local part of a JID, splits it at characters '.', '_', and '-',
    /// capitalizes the first letter of each resulting segment, and then joins them with spaces
    /// to create a formatted display name.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// use prose_core_client::util::StringExt;
    ///
    /// let jid_username = "john_doe";
    /// assert_eq!(jid_username.capitalized_display_name(), "John Doe");
    ///
    /// let jid_username = "jane.doe";
    /// assert_eq!(jid_username.capitalized_display_name(), "Jane Doe");
    /// ```
    ///
    fn capitalized_display_name(&self) -> String;

    /// Slices a String while respecting its boundaries.
    fn safe_slice(&self, range: impl RangeBounds<usize>) -> Option<&str>;

    /// Trims the leading " >" of a quote and the trailing whitespace.
    fn trimmed_quote(&self) -> String;
}

impl<T> StringExt for T
where
    T: AsRef<str>,
{
    // Source: https://stackoverflow.com/a/38406885
    fn to_uppercase_first_letter(&self) -> String {
        let mut c = self.as_ref().chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    fn capitalized_display_name(&self) -> String {
        self.as_ref()
            .split_terminator(&['.', '_', '-'][..])
            .map(|s| s.to_uppercase_first_letter())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn safe_slice(&self, range: impl RangeBounds<usize>) -> Option<&str> {
        let s = self.as_ref();

        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&end) => end + 1,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => s.len(),
        };

        if start <= end && s.is_char_boundary(start) && s.is_char_boundary(end) {
            Some(&s[start..end])
        } else {
            None
        }
    }

    fn trimmed_quote(&self) -> String {
        self.as_ref()
            .trim_end()
            .lines()
            .map(|line| line.strip_prefix("> ").unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        assert_eq!("abc".capitalized_display_name(), "Abc");
        assert_eq!("jane-doe".capitalized_display_name(), "Jane Doe");
        assert_eq!("jane.doe".capitalized_display_name(), "Jane Doe");
        assert_eq!("jane_doe".capitalized_display_name(), "Jane Doe");
    }

    #[test]
    fn test_safe_slice() {
        let s = "Hello World!";

        // Test valid slices
        assert_eq!(Some("Hello"), s.safe_slice(0..5));
        assert_eq!(Some("World"), s.safe_slice(6..11));
        assert_eq!(Some("Hello World!"), s.safe_slice(0..s.len()));

        // Test empty slice
        assert_eq!(s.safe_slice(5..5), Some(""));

        // Test out of bounds
        assert_eq!(None, s.safe_slice(10..25));
        assert_eq!(None, s.safe_slice(25..30));
        assert_eq!(None, s.safe_slice(0..s.len() + 1));
        assert_eq!(None, s.safe_slice(3..1));

        // Test different range types
        assert_eq!(Some("Hello"), s.safe_slice(..5));
        assert_eq!(Some("World!"), s.safe_slice(6..));
        assert_eq!(Some("Hello World!"), s.safe_slice(..));

        // Test invalid UTF-8 boundaries
        let s = "Hello, 🌍!";
        // Correct slicing at character boundaries
        assert_eq!(Some("Hello, "), s.safe_slice(0..7));
        assert_eq!(Some("🌍"), s.safe_slice(7..11));
        assert_eq!(None, s.safe_slice(7..8)); // Invalid boundary inside the UTF-8 character "🌍"
    }

    #[test]
    fn test_trimmed_quote() {
        assert_eq!(
            "Line 1\nLine 2\nLine 3".to_string(),
            "> Line 1\n> Line 2\nLine 3".trimmed_quote()
        );
        assert_eq!("Line 1".to_string(), "Line 1".trimmed_quote());
        assert_eq!("".to_string(), "".trimmed_quote());
    }
}
