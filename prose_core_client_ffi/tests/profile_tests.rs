use base64;
use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::HandlerBucketExt;
use prose_core_client_ffi::test_helpers::StrExt;
use prose_core_client_ffi::{Account, Result, XMPPAvatarData, XMPPAvatarMetadataInfo, XMPPImage};
use std::str::FromStr;

#[test]
fn test_sets_avatar_image() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    let image_data = base64::decode("iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==").unwrap();

    account.profile.set_avatar_image(
        "my_request",
        XMPPImage::new(image_data, "image/png", 600, 600),
    )?;

    assert_eq!(
        stanzas.stanza_at_index(0).to_text()?,
        r#"
        <iq id="id_1" type="set">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <publish node="urn:xmpp:avatar:data">
                    <item id="c1fc608fe89995e52457da8364672061af949a94">
                        <data xmlns="urn:xmpp:avatar:data">
                            iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==
                        </data>
                    </item>
                </publish>
            </pubsub>
        </iq>
        "#
        .to_xml_result_string()
    );

    stanzas.clear();

    handlers.send_stanza_str(
        r#"
        <iq id="id_1" type="result" to="test@prose.org">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <publish node="urn:xmpp:avatar:data">
                    <item id="c1fc608fe89995e52457da8364672061af949a94"/>
                </publish>
            </pubsub>
        </iq>"#,
    );

    assert_eq!(
        stanzas.stanza_at_index(0).to_text()?,
        r#"
        <iq id="id_2" type="set">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <publish node="urn:xmpp:avatar:metadata">
                    <item id="c1fc608fe89995e52457da8364672061af949a94">
                        <metadata xmlns="urn:xmpp:avatar:metadata">
                            <info height="600" bytes="139" id="c1fc608fe89995e52457da8364672061af949a94" type="image/png" width="600"/>
                        </metadata>
                    </item>
                </publish>
            </pubsub>
        </iq>
        "#
            .to_xml_result_string()
    );

    observer
        .lock()
        .unwrap()
        .expect_did_set_avatar_image(
            |arg| arg.partial_eq("my_request"),
            |arg| arg.partial_eq("c1fc608fe89995e52457da8364672061af949a94"),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <iq id="id_2" type="result" to="test@prose.org">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <publish node="urn:xmpp:avatar:metadata">
                    <item id="c1fc608fe89995e52457da8364672061af949a94"/>
                </publish>
            </pubsub>
        </iq>
        "#,
    );

    Ok(())
}

#[test]
fn test_load_latest_metadata() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    account
        .profile
        .load_latest_avatar_metadata("my-request", &BareJid::from_str("a@prose.org").unwrap())?;

    assert_eq!(
        stanzas.stanza_at_index(0).to_text()?,
        r#"
        <iq id="id_1" type="get" to="a@prose.org">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="urn:xmpp:avatar:metadata" max_items="1"/>
            </pubsub>
        </iq>
        "#
        .to_xml_result_string()
    );

    let expected_avatar_meta_data = vec![XMPPAvatarMetadataInfo::new(
        "111f4b3c50d7b0df729d299bc6f8e9ef9066971f",
        None,
        Some(12345),
        Some(64),
        Some(64),
        Some("image/png"),
    )];

    observer
        .lock()
        .unwrap()
        .expect_did_load_avatar_metadata(
            |arg| arg.partial_eq("my-request"),
            |arg| arg.partial_eq(BareJid::from_str("a@prose.org").unwrap()),
            |arg| arg.partial_eq(expected_avatar_meta_data),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <iq id="id_1" type="result" to="marc@prose.org/chat_thingy">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="urn:xmpp:avatar:metadata">
                    <item id="c1fc608fe89995e52457da8364672061af949a94">
                        <metadata xmlns="urn:xmpp:avatar:metadata">
                            <info bytes="12345" height="64" id="111f4b3c50d7b0df729d299bc6f8e9ef9066971f" type="image/png" width="64"/>
                        </metadata>
                    </item>
                </items>
            </pubsub>
        </iq>"#,
    );

    Ok(())
}

#[test]
fn test_load_latest_metadata_with_failure() -> Result<()> {
    let (account, handlers, _, observer) = Account::connected();

    account
        .profile
        .load_latest_avatar_metadata("my-request", &BareJid::from_str("a@prose.org").unwrap())?;

    observer
        .lock()
        .unwrap()
        .expect_did_load_avatar_metadata(
            |arg| arg.partial_eq("my-request"),
            |arg| arg.partial_eq(BareJid::from_str("a@prose.org").unwrap()),
            |arg| arg.partial_eq::<Vec<XMPPAvatarMetadataInfo>>(Vec::new()),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <iq id="id_1" type="error" to="test@prose.org/ci" from="a@prose.org">
            <error type="cancel"><item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas"/></error>
        </iq>"#,
    );

    Ok(())
}

#[test]
fn test_loads_image() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    account.profile.load_avatar_image(
        "my-request",
        &BareJid::from_str("a@prose.org").unwrap(),
        "image-id",
    )?;

    assert_eq!(
        stanzas.stanza_at_index(0).to_text()?,
        r#"
        <iq id="id_1" type="get" to="a@prose.org">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="urn:xmpp:avatar:data">
                    <item id="image-id"/>
                </items>
            </pubsub>
        </iq>
        "#
        .to_xml_result_string()
    );

    let image_data = base64::decode("iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==").unwrap();
    let expected_avatar_data = XMPPAvatarData::new(image_data);

    observer
        .lock()
        .unwrap()
        .expect_did_load_avatar_image(
            |arg| arg.partial_eq("my-request"),
            |arg| arg.partial_eq(BareJid::from_str("a@prose.org").unwrap()),
            |arg| arg.partial_eq(Some(expected_avatar_data)),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <iq id="id_1" type="result" to="marc@prose.org/chat_thingy">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="urn:xmpp:avatar:data">
                    <item id="c1fc608fe89995e52457da8364672061af949a94">
                        <data xmlns="urn:xmpp:avatar:data">iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==</data>
                    </item>
                </items>
            </pubsub>
        </iq>"#,
    );

    // Should be ignored
    handlers.send_stanza_str(
        r#"
        <iq id="id_1" type="result" to="marc@prose.org/chat_thingy">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="urn:xmpp:avatar:data">
                    <item id="c1fc608fe89995e52457da8364672061af949a94">
                        <data xmlns="urn:xmpp:avatar:data">iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==</data>
                    </item>
                </items>
            </pubsub>
        </iq>"#,
    );

    Ok(())
}

#[test]
fn test_notifies_observer_about_avatar_changes() -> Result<()> {
    let (_, handlers, _, observer) = Account::connected();

    let expected_metadata = vec![
        XMPPAvatarMetadataInfo::new(
            "111f4b3c50d7b0df729d299bc6f8e9ef9066971f",
            None,
            Some(12345),
            Some(64),
            Some(64),
            Some("image/png"),
        ),
        XMPPAvatarMetadataInfo::new(
            "e279f80c38f99c1e7e53e262b440993b2f7eea57",
            Some("http://avatars.example.org/happy.png"),
            Some(12345),
            Some(64),
            Some(64),
            Some("image/png"),
        ),
        XMPPAvatarMetadataInfo::new(
            "357a8123a30844a3aa99861b6349264ba67a5694",
            Some("http://avatars.example.org/happy.gif"),
            Some(23456),
            Some(64),
            Some(64),
            Some("image/gif"),
        ),
        XMPPAvatarMetadataInfo::new(
            "03a179fe37bd5d6bf9c2e1e592a14ae7814e31da",
            Some("http://avatars.example.org/happy.mng"),
            Some(78912),
            Some(64),
            Some(64),
            Some("image/mng"),
        ),
    ];

    observer
        .lock()
        .unwrap()
        .expect_did_receive_updated_avatar_metadata(
            |arg| arg.partial_eq(BareJid::from_str("b@prose.org").unwrap()),
            |arg| arg.partial_eq(expected_metadata),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <message from="b@prose.org">
            <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                    <item id="111f4b3c50d7b0df729d299bc6f8e9ef9066971f">
                        <metadata xmlns="urn:xmpp:avatar:metadata">
                            <info bytes="12345" height="64" id="111f4b3c50d7b0df729d299bc6f8e9ef9066971f" type="image/png" width="64"/>
                            <info bytes="12345" height="64" id="e279f80c38f99c1e7e53e262b440993b2f7eea57" type="image/png" url="http://avatars.example.org/happy.png" width="64"/>
                            <info bytes="23456" height="64" id="357a8123a30844a3aa99861b6349264ba67a5694" type="image/gif" url="http://avatars.example.org/happy.gif" width="64"/>
                            <info bytes="78912" height="64" id="03a179fe37bd5d6bf9c2e1e592a14ae7814e31da" type="image/mng" url="http://avatars.example.org/happy.mng" width="64"/>
                        </metadata>
                    </item>
                </items>
            </event>
            <addresses xmlns="http://jabber.org/protocol/address">
                <address type="replyto" jid="b@prose.org/ci"/>
            </addresses>
        </message>
        "#,
    );

    Ok(())
}
