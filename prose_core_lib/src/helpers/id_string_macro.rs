macro_rules! id_string {
    ($t:ident) => {
        #[derive(Debug, Eq, PartialEq, Hash, Clone, serde::Serialize, serde::Deserialize)]
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

        impl std::str::FromStr for $t {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($t(s.to_string()))
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

pub(crate) use id_string;
