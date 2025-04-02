// prose-core-client/prose-core-integration-tests
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::helpers::TestClient;
use crate::{event, recv};
use pretty_assertions::assert_eq;
use prose_core_client::domain::shared::models::{AvatarId, ServerId};
use prose_core_client::domain::workspace::models::WorkspaceIcon;
use prose_core_client::dtos::{UserId, WorkspaceInfo};
use prose_core_client::{server_id, user_id, ClientEvent};
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_receives_server_vcard() -> anyhow::Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    assert_eq!(
        WorkspaceInfo {
            name: "Prose Org".to_string(),
            icon: None,
            accent_color: None,
        },
        client.workspace.load_workspace_info().await?
    );

    client.push_ctx([("SERVER_ID", "prose.org".to_string())]);

    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{SERVER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:ietf:params:xml:ns:vcard-4.0">
                  <item id="{{SERVER_ID}}" publisher="{{SERVER_ID}}">
                      <vcard xmlns='urn:ietf:params:xml:ns:vcard-4.0'>
                        <fn><text>My Prose Server</text></fn>
                        <kind><text>application</text></kind>
                        <x-accent-color><text>#ff00ff</text></x-accent-color>
                      </vcard>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(client, ClientEvent::WorkspaceInfoChanged);
    }
    client.receive_next().await;

    assert_eq!(
        WorkspaceInfo {
            name: "My Prose Server".to_string(),
            icon: None,
            accent_color: Some("#ff00ff".to_string()),
        },
        client.workspace.load_workspace_info().await?
    );

    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{SERVER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                  <item id="their-avatar-id">
                    <metadata xmlns="urn:xmpp:avatar:metadata">
                      <info bytes="20000" height="400" id="their-avatar-id" type="image/gif" width="400" />
                    </metadata>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(client, ClientEvent::WorkspaceIconChanged);
    }
    client.receive_next().await;

    assert_eq!(
        WorkspaceInfo {
            name: "My Prose Server".to_string(),
            icon: Some(WorkspaceIcon {
                id: AvatarId::from_str_unchecked("their-avatar-id"),
                owner: server_id!("prose.org"),
                mime_type: "image/gif".to_string(),
            }),
            accent_color: Some("#ff00ff".to_string()),
        },
        client.workspace.load_workspace_info().await?
    );

    client.pop_ctx();

    Ok(())
}
