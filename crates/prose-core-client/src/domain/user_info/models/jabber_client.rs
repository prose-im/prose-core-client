// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::StringExt;

pub const PROSE_IM_NODE: &str = "https://prose.org";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JabberClient(Url);

impl JabberClient {
    pub fn is_prose(&self) -> bool {
        self.0.as_str().strip_suffix("/") == Some(PROSE_IM_NODE)
    }
}

impl Display for JabberClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Some(domain) = self.0.domain() else {
            return f.write_str("<unknown client>");
        };

        let name = match domain {
            "blabber.im" => "Blabber",
            "c0nnect.de" => "C0nnect",
            "cheogram.com" => "Cheogram",
            "conversations.im" => "Conversations",
            "dino.im" => "Dino",
            "gajim.org" => "Gajim",
            "github.com/geobra/harbour-shmoose" => "Shmoose",
            "mcabber.com/caps" => "mcabber",
            "monal-im.org/" => "Monal",
            "monocles.eu" => "Monocles",
            "movim.eu" => "Movim",
            "pidgin.im" => "Pidgin",
            "poez.io" => "Poezio",
            "profanity-im.github.io" => "Profanity",
            "prose.org" => "Prose",
            "psi-plus.com" => "Psi+",
            "tigase.org" => "Beagle",
            _ => {
                let domain_name = domain.strip_prefix("www.").unwrap_or(domain);

                let name = domain_name
                    .find(".")
                    .map(|idx| &domain_name[..idx])
                    .unwrap_or(domain_name)
                    .split("-")
                    .map(|part| part.to_uppercase_first_letter())
                    .join(" ");

                return f.write_str(if !name.is_empty() {
                    &name
                } else {
                    "<unknown client>"
                });
            }
        };

        f.write_str(name)
    }
}

impl FromStr for JabberClient {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_display() -> Result<(), url::ParseError> {
        assert_eq!(
            "Psi+",
            "http://psi-plus.com"
                .parse::<JabberClient>()?
                .to_string()
                .as_str()
        );
        assert_eq!(
            "Jibber Jabber",
            "http://wwW.jibber-jabber.co.uk"
                .parse::<JabberClient>()?
                .to_string()
                .as_str()
        );
        // ¯\(°_o)/¯
        assert_eq!(
            "Subdomain",
            "http://subdomain.jibber-jabber.com"
                .parse::<JabberClient>()?
                .to_string()
                .as_str()
        );
        assert_eq!(
            "<unknown client>",
            "http://127.0.0.1"
                .parse::<JabberClient>()?
                .to_string()
                .as_str()
        );
        Ok(())
    }

    #[test]
    fn test_is_prose() -> Result<(), url::ParseError> {
        assert_eq!(
            true,
            "https://prose.org".parse::<JabberClient>()?.is_prose()
        );
        assert_eq!(
            false,
            "https://blabber.im".parse::<JabberClient>()?.is_prose()
        );
        Ok(())
    }
}
