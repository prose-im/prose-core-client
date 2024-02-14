// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

// Source: https://github.com/novacrazy/serde_shims/blob/master/mime/src/lib.rs

use std::fmt;
use std::str::FromStr;

use mime::Mime;
use serde::{de, Deserializer, Serializer};

pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(mime.as_ref())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Mime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a valid MIME type")
        }

        fn visit_str<E>(self, value: &str) -> Result<Mime, E>
        where
            E: de::Error,
        {
            Mime::from_str(value).or_else(|e| Err(E::custom(format!("{}", e))))
        }
    }

    deserializer.deserialize_str(Visitor)
}
