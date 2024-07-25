// prose-core-client/prose-utils
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_export]
macro_rules! id_string {
    ($(#[$meta:meta])* $t:ident) => {
        $(#[$meta])*
        #[derive(Debug, Eq, PartialEq, Hash, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $t(String);

        impl $t {
            #[allow(dead_code)]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl<T> From<T> for $t
        where
            T: Into<String>,
        {
            fn from(s: T) -> $t {
                $t(s.into())
            }
        }

        impl AsRef<str> for $t {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::borrow::Borrow<str> for $t {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl std::str::FromStr for $t {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($t(s.to_string()))
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl minidom::IntoAttributeValue for $t {
            fn into_attribute_value(self) -> Option<String> {
                Some(self.into_inner())
            }
        }
    };
}
