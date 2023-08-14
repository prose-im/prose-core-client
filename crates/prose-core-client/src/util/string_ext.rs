// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub trait StringExt {
    fn to_uppercase_first_letter(&self) -> String;
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
}
