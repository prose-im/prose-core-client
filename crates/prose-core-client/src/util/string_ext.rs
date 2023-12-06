// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
    /// ```
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
}
