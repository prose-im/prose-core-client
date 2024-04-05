// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{PubSubEventType, ServerEvent, UserDeviceEvent};
use prose_core_client::domain::encryption::models::{Device, DeviceId, DeviceList};
use prose_core_client::domain::shared::models::UserId;
use prose_core_client::test::parse_xml;
use prose_core_client::user_id;
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_added_or_updated_items() -> Result<()> {
    // Notification With Payload (https://xmpp.org/extensions/xep-0060.html#publisher-publish-success-withpayload)
    let events =
      parse_xml(
        r#"
        <message xmlns="jabber:client" from="valerian@prose.org" id="LPc3MKmFSDc2bYhL8X-n_5Nz" to="marc@prose.org" type="headline">
          <event xmlns="http://jabber.org/protocol/pubsub#event">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current">
                <list xmlns="eu.siacs.conversations.axolotl">
                  <device id="1637" />
                  <device id="15364" />
                  <device id="14085" label="Some label" />
                </list>
              </item>
            </items>
          </event>
        </message>
      "#,
      )
          .await?;

    assert_eq!(
        events,
        vec![ServerEvent::UserDevice(UserDeviceEvent {
            user_id: user_id!("valerian@prose.org"),
            r#type: PubSubEventType::AddedOrUpdated {
                items: vec![DeviceList {
                    devices: vec![
                        Device {
                            id: DeviceId::from(1637),
                            label: None,
                        },
                        Device {
                            id: DeviceId::from(15364),
                            label: None,
                        },
                        Device {
                            id: DeviceId::from(14085),
                            label: Some("Some label".to_string()),
                        }
                    ]
                }]
            },
        })]
    );

    Ok(())
}
