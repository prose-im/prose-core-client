macro_rules! stanza_base_inner {
    ($t:ident) => {
        use std::fmt;
        use std::str::FromStr;
        use $crate::stanza::StanzaBase;

        impl<'a> StanzaBase for $t<'a> {
            fn stanza(&self) -> &libstrophe::Stanza {
                &self.stanza
            }
            fn stanza_mut(&mut self) -> &mut libstrophe::Stanza {
                self.stanza.to_mut()
            }
            fn stanza_owned(self) -> libstrophe::Stanza {
                self.stanza.into_owned()
            }
        }

        #[allow(dead_code)]
        impl<'a> $t<'a> {
            pub(crate) fn into_inner(self) -> StanzaCow<'a> {
                self.stanza
            }
        }

        impl<'a> FromStr for $t<'a> {
            type Err = ();

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok($t {
                    stanza: libstrophe::Stanza::from_str(s).into(),
                })
            }
        }

        impl<'a> From<&'a libstrophe::Stanza> for $t<'a> {
            fn from(stanza: &'a libstrophe::Stanza) -> Self {
                $t {
                    stanza: stanza.into(),
                }
            }
        }

        impl<'a> From<libstrophe::Stanza> for $t<'a> {
            fn from(stanza: libstrophe::Stanza) -> Self {
                $t {
                    stanza: stanza.into(),
                }
            }
        }

        impl<'a> From<libstrophe::StanzaRef<'a>> for $t<'a> {
            fn from(stanza: libstrophe::StanzaRef<'a>) -> Self {
                $t {
                    stanza: stanza.into(),
                }
            }
        }

        #[allow(dead_code)]
        impl<'a> $t<'a> {
            pub fn clone<'b>(&self) -> $t<'b> {
                (*self.stanza).clone().into()
            }
        }

        impl<'a> fmt::Display for $t<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.stanza().to_string())
            }
        }

        impl<'a> fmt::Debug for $t<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.stanza().to_string())
            }
        }
    };
}

pub(crate) use stanza_base_inner;

#[macro_export]
macro_rules! stanza_base {
    (Stanza) => {
        $crate::helpers::stanza_base_macro::stanza_base_inner!(Stanza);
    };
    ($t:ident) => {
        use $crate::stanza::Stanza;

        $crate::helpers::stanza_base_macro::stanza_base_inner!($t);

        impl<'a> From<Stanza<'a>> for $t<'a> {
            fn from(stanza: Stanza<'a>) -> Self {
                $t {
                    stanza: stanza.into_inner(),
                }
            }
        }
    };
}

pub use stanza_base;
