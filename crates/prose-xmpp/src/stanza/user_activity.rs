// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::ns;
use crate::util::ElementExt;
use anyhow::Context;
use anyhow::Result;
use minidom::Element;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UserActivity {
    pub activity: Option<Activity>,
    pub text: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Activity {
    pub general: activity::General,
    pub specific: Option<activity::Specific>,
}

impl TryFrom<Element> for UserActivity {
    type Error = anyhow::Error;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("activity", ns::USER_ACTIVITY)?;

        let mut user_activity = UserActivity::default();

        for child in root.children() {
            match child.name() {
                "text" => user_activity.text = Some(child.text()),
                _ => user_activity.activity = Some(Activity::try_from(child.clone())?),
            }
        }

        Ok(user_activity)
    }
}

impl From<UserActivity> for Element {
    fn from(value: UserActivity) -> Self {
        Element::builder("activity", ns::USER_ACTIVITY)
            .append_all(value.activity)
            .append_all(
                value
                    .text
                    .map(|text| Element::builder("text", ns::JABBER_CLIENT).append(text)),
            )
            .build()
    }
}

impl TryFrom<Element> for Activity {
    type Error = anyhow::Error;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        let mut activity = Activity {
            general: activity::General::from_str(root.name()).context(format!(
                "Encountered unexpected general activity {}",
                root.name()
            ))?,
            specific: None,
        };

        for child in root.children() {
            let mut specific = activity::Specific::from_str(child.name()).context(format!(
                "Encountered unexpected specific activity {}",
                child.name()
            ))?;

            if let activity::Specific::Other(_) = &specific {
                if !child.text().is_empty() {
                    specific = activity::Specific::Other(Some(child.text()))
                }
            }

            activity.specific = Some(specific);
        }

        Ok(activity)
    }
}

impl From<Activity> for Element {
    fn from(value: Activity) -> Self {
        Element::builder(value.general.to_string(), ns::USER_ACTIVITY)
            .append_all(value.specific)
            .build()
    }
}

impl From<activity::Specific> for Element {
    fn from(value: activity::Specific) -> Self {
        let text = if let activity::Specific::Other(Some(text)) = &value {
            Some(text.clone())
        } else {
            None
        };

        Element::builder(value.to_string(), ns::JABBER_CLIENT)
            .append_all(text)
            .build()
    }
}

pub mod activity {
    use strum_macros::{Display, EnumString};

    #[derive(Debug, PartialEq, Display, EnumString, Clone)]
    #[strum(serialize_all = "snake_case")]
    pub enum General {
        DoingChores,
        Drinking,
        Eating,
        Exercising,
        Grooming,
        HavingAppointment,
        Inactive,
        Relaxing,
        Talking,
        Traveling,
        Undefined,
        Working,
    }

    #[derive(Debug, PartialEq, Display, EnumString, Clone)]
    #[strum(serialize_all = "snake_case")]
    pub enum Specific {
        AtTheSpa,
        BrushingTeeth,
        BuyingGroceries,
        Cleaning,
        Coding,
        Commuting,
        Cooking,
        Cycling,
        Dancing,
        DayOff,
        DoingMaintenance,
        DoingTheDishes,
        DoingTheLaundry,
        Driving,
        Fishing,
        Gaming,
        Gardening,
        GettingAHaircut,
        GoingOut,
        HangingOut,
        HavingABeer,
        HavingASnack,
        HavingBreakfast,
        HavingCoffee,
        HavingDinner,
        HavingLunch,
        HavingTea,
        Hiding,
        Hiking,
        InACar,
        InAMeeting,
        InRealLife,
        Jogging,
        OnABus,
        OnAPlane,
        OnATrain,
        OnATrip,
        OnThePhone,
        OnVacation,
        OnVideoPhone,
        Other(Option<String>),
        Partying,
        PlayingSports,
        Praying,
        Reading,
        Rehearsing,
        Running,
        RunningAnErrand,
        ScheduledHoliday,
        Shaving,
        Shopping,
        Skiing,
        Sleeping,
        Smoking,
        Socializing,
        Studying,
        Sunbathing,
        Swimming,
        TakingABath,
        TakingAShower,
        Thinking,
        Walking,
        WalkingTheDog,
        WatchingAMovie,
        WatchingTv,
        WorkingOut,
        Writing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stanza::user_activity::activity::{General, Specific};
    use std::str::FromStr;

    #[test]
    fn test_deserialize_activity() -> Result<()> {
        let xml = r#"<activity xmlns='http://jabber.org/protocol/activity'>
            <relaxing>
                <partying/>
            </relaxing>
            <text xml:lang='en'>My nurse&apos;s birthday!</text>
        </activity>
        "#;

        let elem = Element::from_str(xml)?;
        let user_activity = UserActivity::try_from(elem)?;

        assert_eq!(
            user_activity,
            UserActivity {
                activity: Some(Activity {
                    general: General::Relaxing,
                    specific: Some(Specific::Partying),
                }),
                text: Some(String::from("My nurse's birthday!")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_empty_activity() -> Result<()> {
        let xml = r#"<activity xmlns='http://jabber.org/protocol/activity'/>"#;

        let elem = Element::from_str(xml)?;
        let user_activity = UserActivity::try_from(elem)?;

        assert_eq!(user_activity, UserActivity::default());

        Ok(())
    }

    #[test]
    fn test_deserialize_other_activity_with_emoji() -> Result<()> {
        let xml = r#"<activity xmlns="http://jabber.org/protocol/activity">
            <undefined>
                <other>🌮</other>
            </undefined>
            <text>Eating lunch</text>
        </activity>"#;

        let elem = Element::from_str(xml)?;
        let user_activity = UserActivity::try_from(elem)?;

        assert_eq!(
            user_activity,
            UserActivity {
                activity: Some(Activity {
                    general: General::Undefined,
                    specific: Some(Specific::Other(Some(String::from("🌮")))),
                }),
                text: Some(String::from("Eating lunch")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_other_empty_activity() -> Result<()> {
        let xml = r#"<activity xmlns="http://jabber.org/protocol/activity">
            <undefined>
                <other/>
            </undefined>
            <text>Eating lunch</text>
        </activity>"#;

        let elem = Element::from_str(xml)?;
        let user_activity = UserActivity::try_from(elem)?;

        assert_eq!(
            user_activity,
            UserActivity {
                activity: Some(Activity {
                    general: General::Undefined,
                    specific: Some(Specific::Other(None)),
                }),
                text: Some(String::from("Eating lunch")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_activity() -> Result<()> {
        let activity = UserActivity {
            activity: Some(Activity {
                general: General::Relaxing,
                specific: Some(Specific::Partying),
            }),
            text: Some(String::from("My nurse's birthday!")),
        };

        assert_eq!(
            UserActivity::try_from(Element::from(activity.clone()))?,
            activity
        );
        Ok(())
    }

    #[test]
    fn test_serialize_other_activity() -> Result<()> {
        let activity = UserActivity {
            activity: Some(Activity {
                general: General::Undefined,
                specific: Some(Specific::Other(Some(String::from("🌮")))),
            }),
            text: Some(String::from("Eating lunch")),
        };

        assert_eq!(
            UserActivity::try_from(Element::from(activity.clone()))?,
            activity
        );
        Ok(())
    }
}
