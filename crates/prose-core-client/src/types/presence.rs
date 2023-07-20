use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use xmpp_parsers::presence;

#[derive(Serialize, Deserialize, Default)]
pub struct Presence {
    pub kind: Option<Type>,
    pub show: Option<Show>,
    pub status: Option<String>,
}

pub struct Type(pub presence::Type);
pub struct Show(pub presence::Show);

impl From<presence::Type> for Type {
    fn from(value: presence::Type) -> Self {
        Type(value)
    }
}

impl From<presence::Show> for Show {
    fn from(value: presence::Show) -> Self {
        Show(value)
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        use presence::Type;

        match self.0 {
            Type::None => "",
            Type::Error => "error",
            Type::Probe => "probe",
            Type::Subscribe => "subscribe",
            Type::Subscribed => "subscribed",
            Type::Unavailable => "unavailable",
            Type::Unsubscribe => "unsubscribe",
            Type::Unsubscribed => "unsubscribed",
        }
        .to_string()
    }
}

impl FromStr for Type {
    type Err = <presence::Type as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Type(presence::Type::from_str(s)?))
    }
}

impl ToString for Show {
    fn to_string(&self) -> String {
        use presence::Show;

        match self.0 {
            Show::Away => "away",
            Show::Chat => "chat",
            Show::Dnd => "dnd",
            Show::Xa => "xa",
        }
        .to_string()
    }
}

impl FromStr for Show {
    type Err = <presence::Show as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Show(presence::Show::from_str(s)?))
    }
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Type::from_str(&value).map_err(de::Error::custom)?)
    }
}

impl Serialize for Show {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Show {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Show::from_str(&value).map_err(de::Error::custom)?)
    }
}
