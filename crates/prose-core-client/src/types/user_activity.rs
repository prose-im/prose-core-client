// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::bail;
use anyhow::Result;
use prose_xmpp::stanza::user_activity::activity;
use prose_xmpp::stanza::UserActivity as XMPPUserActivity;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub emoji: String,
    pub status: Option<String>,
}

impl TryFrom<XMPPUserActivity> for UserActivity {
    type Error = anyhow::Error;

    fn try_from(value: XMPPUserActivity) -> Result<Self> {
        // Notice: sending with no icon indicates that we are willing to retract \
        //   any previously published activity.
        let emoji = value
            .activity
            .as_ref()
            .and_then(|activity| activity.specific.as_ref())
            .and_then(|specific| specific.to_emoji());

        let Some(emoji) = emoji else {
            bail!("Missing emoji in UserActivity")
        };

        Ok(UserActivity {
            emoji,
            status: value
                .text
                .or(value.activity.map(|activity| activity.general.to_string())),
        })
    }
}

impl From<UserActivity> for XMPPUserActivity {
    fn from(value: UserActivity) -> Self {
        // Notice: as we are using emoji-based icons in order to specify the \
        //   kind of activity, we do not map to a proper RPID there, but \
        //   rather use the 'undefined' unspecified activity general category, \
        //   and the 'other' specific instance, with a text value for the \
        //   icon. Given that the specification is far too limiting in terms \
        //   of available activity categories, we prefer to rely on more \
        //   modern, emoji-based, activities.

        XMPPUserActivity {
            activity: Some(prose_xmpp::stanza::user_activity::Activity {
                general: activity::General::Undefined,
                specific: Some(activity::Specific::Other(Some(value.emoji))),
            }),
            text: value.status,
        }
    }
}

trait ToEmoji {
    fn to_emoji(&self) -> Option<String>;
}

impl ToEmoji for activity::Specific {
    fn to_emoji(&self) -> Option<String> {
        use activity::Specific;

        match self {
            Specific::AtTheSpa => Some("💆".to_string()),
            Specific::BrushingTeeth => Some("🪥".to_string()),
            Specific::BuyingGroceries => Some("🛒".to_string()),
            Specific::Cleaning => Some("🧹".to_string()),
            Specific::Coding => Some("💻".to_string()),
            Specific::Commuting => Some("🚆".to_string()),
            Specific::Cooking => Some("🍳".to_string()),
            Specific::Cycling => Some("🚴".to_string()),
            Specific::Dancing => Some("🕺".to_string()),
            Specific::DayOff => Some("🛌".to_string()),
            Specific::DoingMaintenance => Some("🔧".to_string()),
            Specific::DoingTheDishes => Some("🧽".to_string()),
            Specific::DoingTheLaundry => Some("🧺".to_string()),
            Specific::Driving => Some("🚗".to_string()),
            Specific::Fishing => Some("🎣".to_string()),
            Specific::Gaming => Some("🎮".to_string()),
            Specific::Gardening => Some("🌱".to_string()),
            Specific::GettingAHaircut => Some("💇‍♂️".to_string()),
            Specific::GoingOut => Some("🚶".to_string()),
            Specific::HangingOut => Some("👫".to_string()),
            Specific::HavingABeer => Some("🍺".to_string()),
            Specific::HavingASnack => Some("🍪".to_string()),
            Specific::HavingBreakfast => Some("🥞".to_string()),
            Specific::HavingCoffee => Some("☕️".to_string()),
            Specific::HavingDinner => Some("🍲".to_string()),
            Specific::HavingLunch => Some("🥪".to_string()),
            Specific::HavingTea => Some("🍵".to_string()),
            Specific::Hiding => Some("🙈".to_string()),
            Specific::Hiking => Some("🥾".to_string()),
            Specific::InACar => Some("🚙".to_string()),
            Specific::InAMeeting => Some("📈".to_string()),
            Specific::InRealLife => Some("👥".to_string()),
            Specific::Jogging => Some("🏃".to_string()),
            Specific::OnABus => Some("🚌".to_string()),
            Specific::OnAPlane => Some("✈️".to_string()),
            Specific::OnATrain => Some("🚂".to_string()),
            Specific::OnATrip => Some("🧳".to_string()),
            Specific::OnThePhone => Some("📞".to_string()),
            Specific::OnVacation => Some("🏖️".to_string()),
            Specific::OnVideoPhone => Some("📹".to_string()),
            Specific::Other(emoji) => emoji.clone(),
            Specific::Partying => Some("🎉".to_string()),
            Specific::PlayingSports => Some("⚽".to_string()),
            Specific::Praying => Some("🙏".to_string()),
            Specific::Reading => Some("📖".to_string()),
            Specific::Rehearsing => Some("🎭".to_string()),
            Specific::Running => Some("🏃".to_string()),
            Specific::RunningAnErrand => Some("🏃🛍️".to_string()),
            Specific::ScheduledHoliday => Some("🗓️".to_string()),
            Specific::Shaving => Some("🪒".to_string()),
            Specific::Shopping => Some("🛍️".to_string()),
            Specific::Skiing => Some("⛷️".to_string()),
            Specific::Sleeping => Some("😴".to_string()),
            Specific::Smoking => Some("🚬".to_string()),
            Specific::Socializing => Some("🗣️".to_string()),
            Specific::Studying => Some("📚".to_string()),
            Specific::Sunbathing => Some("🌞".to_string()),
            Specific::Swimming => Some("🏊‍".to_string()),
            Specific::TakingABath => Some("🛀".to_string()),
            Specific::TakingAShower => Some("🚿".to_string()),
            Specific::Thinking => Some("💭".to_string()),
            Specific::Walking => Some("🚶".to_string()),
            Specific::WalkingTheDog => Some("🚶🐶".to_string()),
            Specific::WatchingAMovie => Some("🎥".to_string()),
            Specific::WatchingTv => Some("📺".to_string()),
            Specific::WorkingOut => Some("🏋️‍".to_string()),
            Specific::Writing => Some("✍️".to_string()),
        }
    }
}
