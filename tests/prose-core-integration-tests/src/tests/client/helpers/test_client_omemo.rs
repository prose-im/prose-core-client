// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use minidom::Element;

use prose_core_client::app::deps::DynEncryptionDomainService;
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::encryption::models::Device;
use prose_core_client::domain::encryption::repos::mocks::MockUserDeviceRepository;
use prose_core_client::domain::encryption::services::impls::{
    EncryptionDomainService, EncryptionDomainServiceDependencies,
};
use prose_core_client::domain::encryption::services::mocks::MockUserDeviceService;
use prose_core_client::domain::encryption::services::{
    EncryptionDomainService as EncryptionDomainServiceTrait, IncrementingUserDeviceIdProvider,
};
use prose_core_client::domain::messaging::services::mocks::MockMessagingService;
use prose_core_client::dtos::{DeviceBundle, DeviceId, UserId};
use prose_core_client::infra::encryption::{EncryptionKeysRepository, SessionRepository};
use prose_core_client::infra::general::mocks::StepRngProvider;
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::ConstantTimeProvider;
use prose_core_client::SignalServiceHandle;

use crate::tests::client::helpers::element_ext::ElementExt;
use crate::tests::store;
use crate::{recv, send};

use super::TestClient;

impl TestClient {
    pub fn expect_load_device_list(
        &self,
        user_id: &UserId,
        device_ids: impl IntoIterator<Item = DeviceId>,
    ) {
        let devices = device_ids
            .into_iter()
            .map(|id| format!("<device id='{id}'/>"))
            .collect::<Vec<_>>()
            .join("\n");

        self.push_ctx(
            [
                ("USER_ID".into(), user_id.to_string()),
                ("DEVICES".into(), devices),
            ]
            .into(),
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{USER_ID}}" type="get">
            <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <items node="eu.siacs.conversations.axolotl.devicelist"/>
            </pubsub>
        </iq>
        "#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current">
                <list xmlns="eu.siacs.conversations.axolotl">
                    {{DEVICES}}
                </list>
              </item>
            </items>
          </pubsub>
        </iq>
            "#
        );

        self.pop_ctx();
    }

    pub fn expect_load_device_bundle(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        bundle: Option<DeviceBundle>,
    ) {
        self.push_ctx(
            [
                ("USER_ID".into(), user_id.to_string()),
                ("DEVICE_ID".into(), device_id.as_ref().to_string()),
            ]
            .into(),
        );

        send!(
            self,
            r#"
                <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="get">
                  <pubsub xmlns="http://jabber.org/protocol/pubsub">
                    <items node="eu.siacs.conversations.axolotl.bundles:{{DEVICE_ID}}" />
                  </pubsub>
                </iq>
                "#
        );

        if let Some(bundle) = bundle {
            self.push_ctx(
                [(
                    "BUNDLE".into(),
                    String::from(&Element::from(xmpp_parsers::legacy_omemo::Bundle::from(
                        bundle,
                    ))),
                )]
                .into(),
            );

            recv!(
                self,
                r#"
                <iq xmlns='jabber:client' id="{{ID}}" type="result">
                  <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                    <items node="eu.siacs.conversations.axolotl.bundles:{{DEVICE_ID}}">
                      <item xmlns='http://jabber.org/protocol/pubsub' id="current">
                        {{BUNDLE}}
                      </item>
                    </items>
                  </pubsub>
                </iq>
                "#
            );

            self.pop_ctx();
        } else {
            recv!(
                self,
                r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="error">
              <error type="cancel">
                <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#
            );
        }

        self.pop_ctx()
    }

    pub fn expect_publish_initial_device_bundle(&self) {
        send!(
            self,
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" to="{{USER_ID}}" type="get">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <items node="eu.siacs.conversations.axolotl.bundles:{{USER_DEVICE_ID}}" />
              </pubsub>
            </iq>
            "#
        );
        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="error">
              <error type="cancel">
                <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#
        );

        self.expect_publish_device_bundle(TestClient::initial_device_bundle_xml());
    }

    pub fn expect_publish_device_bundle(&self, bundle_xml: impl Into<String>) {
        self.push_ctx([("DEVICE_BUNDLE".into(), bundle_xml.into())].into());

        send!(
            self,
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" type="set">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="eu.siacs.conversations.axolotl.bundles:{{USER_DEVICE_ID}}">
                  <item id="current">
                    {{DEVICE_BUNDLE}}
                  </item>
                </publish>
                <publish-options>
                  <x xmlns='jabber:x:data' type="submit">
                    <field type="hidden" var="FORM_TYPE">
                      <value>http://jabber.org/protocol/pubsub#publish-options</value>
                    </field>
                    <field var="pubsub#access_model">
                      <value>open</value>
                    </field>
                  </x>
                </publish-options>
              </pubsub>
            </iq>
            "#
        );

        self.pop_ctx();

        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="eu.siacs.conversations.axolotl.bundles:0">
              <item id="current" />
            </publish>
          </pubsub>
        </iq>
        "#
        );
    }
}

impl TestClient {
    pub fn device_id() -> u32 {
        12345
    }

    pub fn their_device_id() -> u32 {
        54321
    }

    pub async fn their_encryption_domain_service(
        their_user_id: UserId,
    ) -> DynEncryptionDomainService {
        let store = store().await.unwrap();

        let encryption_keys_repo = Arc::new(EncryptionKeysRepository::new(store.clone()));
        let session_repo = Arc::new(SessionRepository::new(store.clone()));
        let rng_provider = Arc::new(StepRngProvider::default());
        let encryption_service = Arc::new(SignalServiceHandle::new(
            encryption_keys_repo.clone(),
            session_repo.clone(),
            rng_provider.clone(),
        ));

        let connection_props = ConnectionProperties {
            connection_timestamp: Default::default(),
            connected_jid: their_user_id.with_resource("their_device").unwrap(),
            server_features: Default::default(),
            rooms_caught_up: false,
            decryption_context: None,
        };

        let mut user_device_repo = MockUserDeviceRepository::new();
        {
            let their_user_id = their_user_id.clone();
            user_device_repo
                .expect_get_all()
                .returning(move |_, user_id| {
                    let device = if user_id == &their_user_id {
                        Device {
                            id: TestClient::their_device_id().into(),
                            label: None,
                        }
                    } else {
                        Device {
                            id: TestClient::device_id().into(),
                            label: None,
                        }
                    };
                    Box::pin(async move { Ok(vec![device]) })
                });
        }

        let mut user_device_service = MockUserDeviceService::new();
        user_device_service
            .expect_load_device_bundle()
            .returning(move |user_id, _| {
                let bundle = if user_id == &their_user_id {
                    None
                } else {
                    Some(TestClient::initial_device_bundle())
                };
                Box::pin(async { Ok(bundle) })
            });
        user_device_service
            .expect_publish_device_bundle()
            .once()
            .return_once(|_| Box::pin(async { Ok(()) }));

        let deps = EncryptionDomainServiceDependencies {
            ctx: Arc::new(Default::default()),
            encryption_keys_repo,
            encryption_service,
            message_repo: Arc::new(CachingMessageRepository::new(store.clone())),
            messaging_service: Arc::new(MockMessagingService::new()),
            rng_provider,
            session_repo,
            time_provider: Arc::new(ConstantTimeProvider::ymd(2024, 1, 1)),
            user_device_id_provider: Arc::new(IncrementingUserDeviceIdProvider::new(
                Self::their_device_id(),
            )),
            user_device_repo: Arc::new(user_device_repo),
            user_device_service: Arc::new(user_device_service),
        };

        deps.ctx.set_connection_properties(connection_props);

        let domain_service = Arc::new(EncryptionDomainService::from(deps));
        domain_service.initialize().await.unwrap();

        domain_service
    }

    pub fn initial_device_bundle_xml() -> &'static str {
        r#"
        <bundle xmlns='eu.siacs.conversations.axolotl'>
          <signedPreKeyPublic signedPreKeyId="0">BTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJu</signedPreKeyPublic>
          <signedPreKeySignature>J+lp0UiI/fseBjUZv1xkmXLJN/Jc0jHY67w+grWQBpOow8Vxm11F/D2rGoskGn6Qjlymh5a8kH+PNcg9EtVlhA==</signedPreKeySignature>
          <identityKey>BTQ9Qr1iZH0bYjwm34NOaKoc3g2bCKMzsqyeNihNgaUx</identityKey>
          <prekeys>
            <preKeyPublic preKeyId="1">BW5hMOrNOjAiWAex/RebnNDAq4vFVz30wLGFhBSAdyoy</preKeyPublic>
            <preKeyPublic preKeyId="2">BWdlp0kR2w5GIk46mMi56z+0CjaplyLuK+fCtG2UwHcF</preKeyPublic>
            <preKeyPublic preKeyId="3">BYDostUvT21ikRmdTXthG2E6kGbMZ9SNKiALuNlIlzJU</preKeyPublic>
            <preKeyPublic preKeyId="4">BZQnww6/Hm6QrDR5FwfBT2IxJz5DtEuoElrOG1nSaAt/</preKeyPublic>
            <preKeyPublic preKeyId="5">BWK10ShyKhmlLVgdQtUSNM84CXxecmXffGUd5s7cBn1u</preKeyPublic>
            <preKeyPublic preKeyId="6">BeaODWWJ2DATWC6fLaEcptPuTlStw6M132HeqiF3hbcK</preKeyPublic>
            <preKeyPublic preKeyId="7">BUv0d7f4x8LwQNRedD02feIkzctOMTVpsouMlMwtb3MB</preKeyPublic>
            <preKeyPublic preKeyId="8">BUJ+7apOHtrAqqSvFISsOs2DMVPuACqbHGiPx2PRGN1k</preKeyPublic>
            <preKeyPublic preKeyId="9">BXQtt3vu7YGMA/QmUKGfBR8/vYSacFun8lYsKwTsj7gc</preKeyPublic>
            <preKeyPublic preKeyId="10">BVzsDdKSadSYQLdRg+YVQY2sFyn1JRGwDvp9dh6nqU4w</preKeyPublic>
            <preKeyPublic preKeyId="11">Bbj5U8RizH7ZRHI+GOb+xgSwh56qmLyuxuet9wGMSck2</preKeyPublic>
            <preKeyPublic preKeyId="12">Bbicpsiviwn5JNlHlvDR6QcB7Z8kKueopBaJkpEoq2IQ</preKeyPublic>
            <preKeyPublic preKeyId="13">BeJaHJAY0ZC+lBdPxvh+IGNlMeSQ0+b+chakFQvNCVsa</preKeyPublic>
            <preKeyPublic preKeyId="14">BUR81M1CsLCscKC/lcRLERlrgoS8MciqgvtfdZ38zZEB</preKeyPublic>
            <preKeyPublic preKeyId="15">BUz1MOeF1+lY+XYWAj5y0IgnL1cEnmZ/VH5yfR8UGIRI</preKeyPublic>
            <preKeyPublic preKeyId="16">BWcm1Ix9lWN6nHRCCrlGF5NCgT8HQmcDiwj4sO86Ch41</preKeyPublic>
            <preKeyPublic preKeyId="17">Bc+Hz1+yFh2xC3N3QBjWcag0M8CMGxose+l24SX+3ZcJ</preKeyPublic>
            <preKeyPublic preKeyId="18">Bfu2U0imlpP8Ipr9B+ZY+Mv3QPKJVVMVPAQkttIZ1Fsy</preKeyPublic>
            <preKeyPublic preKeyId="19">BTuuNosn2JYOXfQkoIK/jjGf3eLEx0OaIReg2EmLVTtK</preKeyPublic>
            <preKeyPublic preKeyId="20">BRcvPnUNSefPJQ72W3IycCPn3lyPlgf7MYTgZwazuyJ5</preKeyPublic>
            <preKeyPublic preKeyId="21">BcaBUUfpkTwGcf2BOkPu9PFYmSygTY1l8sa+jgVVQZsc</preKeyPublic>
            <preKeyPublic preKeyId="22">BWfXIXMRNXtsTGQxz6TDGW/56IgT7H6P5qjWfk1lamJL</preKeyPublic>
            <preKeyPublic preKeyId="23">BTdOvUsLR+Yq5D0EjYf+DTUoFnPeQex/2ALz3ARgDSp3</preKeyPublic>
            <preKeyPublic preKeyId="24">BWOBsBbcMid5PxcRVKbSGAHxkysWBiq94oRUk9Ww8ewH</preKeyPublic>
            <preKeyPublic preKeyId="25">BcYiWpju8dMKIIwYHu1sZN0cv7iLlyPJiMgJ+ugu+GtJ</preKeyPublic>
            <preKeyPublic preKeyId="26">BYHnhZO/zyIURS3jZZZ4z4pKMw16vtvU4MuL1wcCI9s+</preKeyPublic>
            <preKeyPublic preKeyId="27">BQednCrsB/dpI1xxO5x2uo9aNfE2v0yroy96ESzSFpNN</preKeyPublic>
            <preKeyPublic preKeyId="28">BaOcL7GZbO7OVmsKup+zFUAurkWlzV/Vw4kuHz4qeAo1</preKeyPublic>
            <preKeyPublic preKeyId="29">Bb6ijLuRdgbB5IMoJMazt4MoOyaLvZgIn1t6b3Uih7xo</preKeyPublic>
            <preKeyPublic preKeyId="30">BdOnjjGijqcSiUncfods6ABmL8nDvCicobGrKmT4OR9X</preKeyPublic>
            <preKeyPublic preKeyId="31">BQBKBAyig6Au8DkudgBrjIG5V3d0E9ksSt7r5T043idC</preKeyPublic>
            <preKeyPublic preKeyId="32">BVd5KMu/1ok/p2uII0UbKbICxohQZHXFkT0FdTVMWRxt</preKeyPublic>
            <preKeyPublic preKeyId="33">BRXEXD+W3K6n5XvcPoTdeBb18OE5Xwnhuo6TUqMlN0ls</preKeyPublic>
            <preKeyPublic preKeyId="34">BXPuTyzTo5Oakw1eH94YjAbbID2ReqMbU+o3CcVJjZAg</preKeyPublic>
            <preKeyPublic preKeyId="35">BSJVZk6RQ2RgcKKv8LLd52fnYARYn0CwE9Kc7Mviy6VJ</preKeyPublic>
            <preKeyPublic preKeyId="36">BSnbDfJ7nznB62YILQt7X2BAkLqfViDMnGpYIlhqoRIE</preKeyPublic>
            <preKeyPublic preKeyId="37">BVOCL0PIvKwPO+vRqNAkXN+mExVNA0BEyVQo7bMpHecz</preKeyPublic>
            <preKeyPublic preKeyId="38">Be9Quy+UDracoGIvwsHyM+E4ADzkFOXO/6GdNmAQuIIi</preKeyPublic>
            <preKeyPublic preKeyId="39">Bbc0eZTfMRGL17ck7T8e65zzGGssDkXn4xMzyJZI8pko</preKeyPublic>
            <preKeyPublic preKeyId="40">BTN9Gqh74n4s5w/4rtK+1frjSUCmew64mXQE6k9ZraVm</preKeyPublic>
            <preKeyPublic preKeyId="41">BSS6qg0bCm0XYVOyFY4xneGwZl7ZpW4PWSBPyBcbOm5C</preKeyPublic>
            <preKeyPublic preKeyId="42">BbyQl32oJb7Ww+WJiHx96iOsRGOND58kGFC26n9BNncJ</preKeyPublic>
            <preKeyPublic preKeyId="43">BSr0kW1t4zyhUXG9UvrI3kzLhZcXTJmd72H/sSme85E1</preKeyPublic>
            <preKeyPublic preKeyId="44">BcvwDF2qziBzSitshuLTTXqK5H06jl9mI1R4MlsNMmxJ</preKeyPublic>
            <preKeyPublic preKeyId="45">BVkkipsRj1PwtYF6ThmCl63WSSQdhiPya+4BbcSMH4o+</preKeyPublic>
            <preKeyPublic preKeyId="46">BYWLty60myPONd7HhNJ/w5GTe0BNGTAY6hLTZD2rJawt</preKeyPublic>
            <preKeyPublic preKeyId="47">BU/pzkOus76D4wmzbOQYVJty2hM1Ggl8/oU3bgBkk6kv</preKeyPublic>
            <preKeyPublic preKeyId="48">BWbaK023ex+ELQIIU+4x4LV5JN0g/QPHNKxgyhSQQm1R</preKeyPublic>
            <preKeyPublic preKeyId="49">BXgIbzLi/58DQVG0fhCVb/KAIBGTJ/3TBDxeaeSY73Vb</preKeyPublic>
            <preKeyPublic preKeyId="50">BW5j8Gvqg9lbGitMBBQOWzLIjPPwN5AAanJwb3tPEtFO</preKeyPublic>
            <preKeyPublic preKeyId="51">BRCgkG4PyYlfp3a9BkZ9kxMfL6YsmV/MKRJGn2iHHa4+</preKeyPublic>
            <preKeyPublic preKeyId="52">BXduHuPo2wR9ASibZh10hcdyWfeTNjZNh3JXGlcth3gd</preKeyPublic>
            <preKeyPublic preKeyId="53">BZTjwBFeUS6JochvRr77yCYM7uKgW2JNhgxytEbn8w8w</preKeyPublic>
            <preKeyPublic preKeyId="54">BZGXBbMul/bjN4BAWqrmLGCRzZi6HO7cBawqU3uUnGdc</preKeyPublic>
            <preKeyPublic preKeyId="55">BX2dy91d7VrdPycYA9m4gGdRa4qCqGV7C1WsSGoYd14v</preKeyPublic>
            <preKeyPublic preKeyId="56">Bf/D5HZSFsGMlssHDs4rwyjMTivArEFrgVtgPQ6tWfUl</preKeyPublic>
            <preKeyPublic preKeyId="57">Ba1409a7gzzDGy5nu8EkusRlDmc6U+iUcAt2aUt8XNMb</preKeyPublic>
            <preKeyPublic preKeyId="58">BYJxZ4d1mpjPVR+WA8ZrJ4pMi0EpawW2UkCMN9/B9cZr</preKeyPublic>
            <preKeyPublic preKeyId="59">BZEjcGY//nftF11PSOhLFqvt1BilS2tC0gaJWd4T1odb</preKeyPublic>
            <preKeyPublic preKeyId="60">BQghhoVlv0IgrGU5Kdmz7EAznadZqjbIKS/ZFVxzNxpU</preKeyPublic>
            <preKeyPublic preKeyId="61">BSoWpVAY5DyTSp8f+OKeIA/QSYUnguylwz1DPUH+/pR+</preKeyPublic>
            <preKeyPublic preKeyId="62">BRfGM9HirZTK7ZU961intuuVDzi6ZCFunBhCd6W4uPYa</preKeyPublic>
            <preKeyPublic preKeyId="63">BatdASXcPtNYuPZnLxqAsbrTbC7rW2K1DxqxqbJ5jClr</preKeyPublic>
            <preKeyPublic preKeyId="64">BWFjL/R1d7HIqGGt0WYO0riB0opQe90M+m8hEtB+Qqp/</preKeyPublic>
            <preKeyPublic preKeyId="65">BTXxBnXcY8VEJ6CdzKT3XT1vK6yG4p0t1ADohbm4U5d4</preKeyPublic>
            <preKeyPublic preKeyId="66">BX4S54W7a3PkTwa5l3Nx2hipU3a+mWKS6muBR1u36dw3</preKeyPublic>
            <preKeyPublic preKeyId="67">BaJybFItFqCFHf16K8bHlZ7bQnXu2uCnX5drEBNbaaQv</preKeyPublic>
            <preKeyPublic preKeyId="68">BQggD4uwNaIK992rhWK8zeHVaohTjCBK0db7wqPDaxBm</preKeyPublic>
            <preKeyPublic preKeyId="69">BT42r4mG7houq7pVxk9FVvOdCnnaneCBJeiurHSixzIq</preKeyPublic>
            <preKeyPublic preKeyId="70">BevBlvecvwp8GZnsqcSsIiu+/TmkzQjGBYUcUKTQu4Em</preKeyPublic>
            <preKeyPublic preKeyId="71">BTyivEThpPh8Man3nMi3AE85z9i/zHQZ3Li1HHseUKs7</preKeyPublic>
            <preKeyPublic preKeyId="72">BVpjURaxPUi/Kxs6LwNBodSGRzwWaNh6xi7uKcd9yQM1</preKeyPublic>
            <preKeyPublic preKeyId="73">BYKO3/4x5yrfQKW5IIgCMS84ygKPkXZ+2ISgKHeu3qQm</preKeyPublic>
            <preKeyPublic preKeyId="74">BWs9WHKITM5ru1lAuDrrZqkxQ+12TQ3qMbslCVrjqclG</preKeyPublic>
            <preKeyPublic preKeyId="75">BaD89nz9FkMxHtVxolvpIJPugIYftlUrmypaYrEgy7R+</preKeyPublic>
            <preKeyPublic preKeyId="76">BQzSE73SgNdy3eoDtJBl81d7WpP/Mm9GFbng5Lt3tqZi</preKeyPublic>
            <preKeyPublic preKeyId="77">BXKH8p5CX0Gz0OG++5GiOehNwtQI+LQfa7CcSHko22Bz</preKeyPublic>
            <preKeyPublic preKeyId="78">BcMkv3kMR5KOKUmXTzuYGjoqTgOqFrRkFb3eDx8M2m8j</preKeyPublic>
            <preKeyPublic preKeyId="79">BY+Ls8DYAyfERrszO1IDtsTUB/PGDd6aQU0Xsj1XrkEM</preKeyPublic>
            <preKeyPublic preKeyId="80">BRUuIgkiADkDpBMwhE6Fmf970gCQOGo5pfi5udq42QFR</preKeyPublic>
            <preKeyPublic preKeyId="81">Bd7QvaMZvXdXAsqLdsefUIYZfU/N3AsDKPo6qM2B95Nz</preKeyPublic>
            <preKeyPublic preKeyId="82">BZMk1ICrP7bRO0dvcJXHlOiUgkHOkLEFNnD54W+Xw19C</preKeyPublic>
            <preKeyPublic preKeyId="83">BUGyidIssUcHX97CuBiWk+2Vw8wZLC3kYkSXVmm0QM0I</preKeyPublic>
            <preKeyPublic preKeyId="84">BRA8FLn/XCHEgF8U+A5VtjQsnBZutCG1zwjIpzFOcJ5u</preKeyPublic>
            <preKeyPublic preKeyId="85">BaR70e/BHGFYc3W0VjtZlHBkwyM4FMUl2rBLfdNSPL8K</preKeyPublic>
            <preKeyPublic preKeyId="86">BfuIl+B3p7QQD963EvnAQFTgQkL83svfIxRewYcP0FIe</preKeyPublic>
            <preKeyPublic preKeyId="87">BUAoeTq/1ijTbutcwVAdZCtFpn63OdJh2OoOtgCjr0Ik</preKeyPublic>
            <preKeyPublic preKeyId="88">BQaP5S8dyV3saHCz8KK2Vu84s0idcuhYZHtvZgSwDOFk</preKeyPublic>
            <preKeyPublic preKeyId="89">BfuJ7eYryvJwNpqlJL6K70+4c+bGY+Cit3oVj5qGfChW</preKeyPublic>
            <preKeyPublic preKeyId="90">BV+5zlrS9kpctmnJuwq8y1ZqVkSrP0MabR6ah49pc8wa</preKeyPublic>
            <preKeyPublic preKeyId="91">BfOp+XYz2XhIFvpv8c93hsS6pTod/GKW3gSDJbfoz5MK</preKeyPublic>
            <preKeyPublic preKeyId="92">BYwA5AC8SvbHMkphvyUSmYWKvlnFshLA+boNkk5MlnU9</preKeyPublic>
            <preKeyPublic preKeyId="93">BZUtWMhq4JEc4SxB38xTjDXmKW0wJ8zqlvXjKaBx6WUt</preKeyPublic>
            <preKeyPublic preKeyId="94">BX1NEmZvierVnBOpl+sF1/uOgWScKQH9JW4jJ2HTcIsT</preKeyPublic>
            <preKeyPublic preKeyId="95">BXaG2in7PDoc/habJg/FolU7IFBT7HU+/UfrIpvKd2dL</preKeyPublic>
            <preKeyPublic preKeyId="96">BbSxylm0+qo3WdFcuiF3hGkUmmSng2laHgGVPyd7TnxE</preKeyPublic>
            <preKeyPublic preKeyId="97">BdvXCYXf5q1PP2ItbE/fk6acIlz9DRMHIbtzbj+O14p8</preKeyPublic>
            <preKeyPublic preKeyId="98">Bfs8o/RIXaxBpNv4XhafzsSdOZ2vaFoIzTa/jciDzB5k</preKeyPublic>
            <preKeyPublic preKeyId="99">Be6Yhor/4ZnvfYIEOJmbQyHFkhDdhtFUKxeuNBD1JKx0</preKeyPublic>
            <preKeyPublic preKeyId="100">BenxPA5A7bu/ZDAl+l7g3qpbcDFY0hHzEUz02RzMEmlH</preKeyPublic>
          </prekeys>
        </bundle>
        "#
    }

    fn initial_device_bundle() -> DeviceBundle {
        DeviceBundle::try_from((
            TestClient::device_id().into(),
            xmpp_parsers::legacy_omemo::Bundle::try_from(
                Element::from_pretty_printed_xml(TestClient::initial_device_bundle_xml()).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap()
    }
}
