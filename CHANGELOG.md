
## What's Changed in 0.1.88

* Move user info/profile repo access into domain service by @nesium
* Merge user_info and user_profile folders by @nesium
* Add vcard-temp parser by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.87...0.1.88)


## What's Changed in 0.1.87

* Introduce invisible availability by @nesium
* Sent messages in MUC rooms would be count as unread by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.86...0.1.87)


## What's Changed in 0.1.86

* Dispatch disconnect events immediately by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.84...0.1.86)


## What's Changed in 0.1.84

* Do not try to access account after logging out by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.83...0.1.84)


## What's Changed in 0.1.83

* Round message timestamps to second precision by @nesium
* Save received message for newly created rooms by @nesium
* Remove redundant trait by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.82...0.1.83)


## What's Changed in 0.1.82

* Reset state, connect & catchup rooms + send MessagesNeedReload event after reconnect by @nesium
* Do not count sent messages as unread in sidebar by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.81...0.1.82)


## What's Changed in 0.1.81

* Provide method to set last read message by @nesium
* Revert to saving StanzaId for last read message by @nesium
* Ping MUC rooms regularly and reconnect if needed (XEP-0410) by @nesium
* Segregate data by account in repositories by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.80...0.1.81)


## What's Changed in 0.1.80

* Channels were not loaded in the web version due to an exception during the connection process by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.79...0.1.80)


## What's Changed in 0.1.79

* Defer deletion of used PreKeys until after catch-up by @nesium
* Await unused future by @nesium
* Save MessageId instead of StanzaId for last unread message to allow marking sent messages as read by @nesium
* Do not show invalid unread count while room is still pending/connecting by @nesium
* Determine server time offset when connecting to improve accuracy of local timestamps by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.78...0.1.79)


## What's Changed in 0.1.78

* Immediately cancel pending futures upon receiving a Disconnected event by @nesium
* Defer processing of offline messages until after complete connection by @nesium
* Improve unread handling and synchronize between clients by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.77...0.1.78)


## What's Changed in 0.1.77

* Let implementor trigger pings and timers by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.76...0.1.77)


## What's Changed in 0.1.76

* Add MessageArchiveDomainService to catch up on conversations by @nesium
* Support full range of MAM queries by @nesium
* Record timestamp when connection was established by @nesium
* Determine supported MAM version by @nesium
* Add local room settings by @nesium
* Include table name in generated index name by @nesium
* Add method to get last received message by @nesium
* Add method to fold rows by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.75...0.1.76)


## What's Changed in 0.1.75

* Delete only messages that belong to a given account by @nesium
* Add delete_all_in_index method by @nesium
* Segregate messages in cache by account and room id by @nesium
* Serialize RoomId identically when used as key or property by @nesium
* Implement KeyType for every &T: KeyType by @nesium
* Support multi-column indexes by @nesium
* Support OMEMO in MUC rooms by @nesium
* Improve event dispatching for sent/received messages by @nesium
* Allow empty message bodies for attachment-only messages by @nesium
* Include mentions in message corrections by @nesium
* Do not send events for changes to our own compose state in MUC rooms by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.74...0.1.75)


## What's Changed in 0.1.74

* Encrypt message updates by @nesium
* Decrypt messages in MUC rooms by @nesium
* Prevent continually trying to load a user’s vCard if none is available by @nesium
* Pass room_id to SidebarDomainService to prevent creating DM sidebar items when receiving a message in a non-anonymous room by @nesium
* Sort imports by @nesium
* Distinguish between NoDevices and NoTrustedDevices errors by @nesium
* Try to repair session when decryption fails by @nesium
* Try to unpublish broken devices only once by @nesium
* Unpublish device if session cannot be started by @nesium
* Request contact name and user info sequentially to have a predictable result by @nesium
* Complete session automatically after receiving prekey message by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.73...0.1.74)


## What's Changed in 0.1.73

* Mark sessions as active/inactive when the corresponding devices disappear/reappear by @nesium
* Move session related methods to separate SessionRepository by @nesium
* Use existing sessions to compile device_infos by @nesium
* Remove EncryptionDirection by @nesium
* Introduce new Session struct, merge identity and session storage by @nesium
* Start OMEMO sessions for own devices lazily by @nesium
* Remove UserDeviceKey::min & max which would lead to incorrect queries by @nesium
* Throw EncryptionError when recipient has no OMEMO devices by @nesium
* Remove start_session from EncryptionDomainService trait by @nesium
* Rename user_device_bundle.rs to device_bundle.rs by @nesium
* Start OMEMO session on demand by @nesium
* Introduce EncryptionError and throw when recipient has no OMEMO devices by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.72...0.1.73)


## What's Changed in 0.1.72

* Re-publish own device list if current device is not included in PubSub message by @nesium
* Handle empty OMEMO messages by @nesium
* Rename EncryptedMessage to EncryptionKey by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.71...0.1.72)


## What's Changed in 0.1.71

* Ignore failures when querying server components on startup by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.70...0.1.71)


## What's Changed in 0.1.70

* Add interface and support for custom JS logger by @nesium
* Trim vCard values which would lead to empty user names by @nesium
* Resolve reaction senders’ names by @nesium
* Include local timezone in entity time request ([#71](https://github.com/prose-im/prose-core-client/issues/71)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.69...0.1.70)


## What's Changed in 0.1.69

* Cache device list, include trust and fingerprint in DeviceDTO by @nesium
* Clean OMEMO related data when clearing cache by @nesium
* Support deleting OMEMO devices by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.68...0.1.69)


## What's Changed in 0.1.67

* feat: Support OMEMO in 1:1 conversations by @nesium in [#70](https://github.com/prose-im/prose-core-client/pull/70)
* Support numeric keys in get_all methods by @nesium
* Immediately insert participant in pending direct message rooms so that it can be accessed in API consumers by @nesium
* Prevent MUC rooms from sending message history by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.66...0.1.67)


## What's Changed in 0.1.66

* Parse private MUC messages and mark them as transient by @nesium
* Do not use id_provider for generating Future IDs by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.65...0.1.66)


## What's Changed in 0.1.65

* Prevent dangling futures when client is disconnected by @nesium
* Update sent reactions in a MUC room by @nesium
* Allow providing a media type when requesting an UploadSlot by @nesium
* Interpret m4a extension as audio/mp4 by @nesium
* Simplify querying PubSub nodes by @nesium
* Implement PubSub event generically ([#19](https://github.com/prose-im/prose-core-client/issues/19)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.64...0.1.65)


## What's Changed in 0.1.64

* Support mentions by @nesium
* Target messages for chat markers via StanzaId in MUC rooms ([#60](https://github.com/prose-im/prose-core-client/issues/60)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.63...0.1.64)


## What's Changed in 0.1.63

* Treat MUC messages sent by us correctly as “sent message” by @nesium
* Improve carbon handling by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.62...0.1.63)


## What's Changed in 0.1.62

* Allow message payloads to target messages by their stanza id ([#60](https://github.com/prose-im/prose-core-client/issues/60)) by @nesium
* Remove assertion that leads to crash when receiving a cached/delayed message on login by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.61...0.1.62)


## What's Changed in 0.1.61

* Do not consume MAM messages in future to join room ([#55](https://github.com/prose-im/prose-core-client/issues/55)) by @nesium
* Separate RoomIds into MUC and non-MUC ([#55](https://github.com/prose-im/prose-core-client/issues/55)) by @nesium
* Increase unread count in channels ([#49](https://github.com/prose-im/prose-core-client/issues/49)) by @nesium
* Update unread count in sidebar ([#49](https://github.com/prose-im/prose-core-client/issues/49)) by @nesium
* Reduce newer cached messages into older messages loaded from server ([#59](https://github.com/prose-im/prose-core-client/issues/59)) by @nesium
* Update MessagesRepository method signature by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.60...0.1.61)


## What's Changed in 0.1.59

* Fill MessageResultSet to a minimum (or more) of guaranteed messages ([#58](https://github.com/prose-im/prose-core-client/issues/58)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.58...0.1.59)


## What's Changed in 0.1.58

* Support loading older messages from MAM ([#58](https://github.com/prose-im/prose-core-client/issues/58)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.57...0.1.58)


## What's Changed in 0.1.57

* Rejoining an already joined group would not bring it back into the sidebar ([#56](https://github.com/prose-im/prose-core-client/issues/56)) by @nesium
* Move RoomEnvelope into dtos module by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.56...0.1.57)


## What's Changed in 0.1.56

* Regression where room topic was not available anymore introduced in #52 by @nesium
* Regression in merging/lookup of participants introduced in #52 ([#54](https://github.com/prose-im/prose-core-client/issues/54)) by @nesium
* Prevent stanzas being handled out of order with the native connector by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.55...0.1.56)


## What's Changed in 0.1.55

* Show current user in participants list when affiliation is less than member ([#53](https://github.com/prose-im/prose-core-client/issues/53)) by @nesium
* Allow RequestFutures to consume XMPPElements ([#52](https://github.com/prose-im/prose-core-client/issues/52)) by @nesium
* Collect message history and subject when joining a MUC room by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.54...0.1.55)


## What's Changed in 0.1.54

* Thumbnails set via Attachment::video_attachment were discarded by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.53...0.1.54)


## What's Changed in 0.1.53

* Support XEP-0385 attachments and thumbnails ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.52...0.1.53)


## What's Changed in 0.1.52

* Do not dispatch event when relaying a message by @nesium
* Dispatch event again when sending a message ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.51...0.1.52)


## What's Changed in 0.1.51

* Add some documentation to wasm bindings ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium
* Support sending/receiving messages with attachments ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium
* Add functionality to request upload slot ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium
* Parse HTTP upload endpoint into server features ([#24](https://github.com/prose-im/prose-core-client/issues/24)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.50...0.1.51)


## What's Changed in 0.1.50

* Improve error handling ([prose-app-web#38](https://github.com/prose-im/prose-app-web/issues/38)) by @nesium
* Introduce MessageMetadata with isRead flag ([#48](https://github.com/prose-im/prose-core-client/issues/48)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.49...0.1.50)


## What's Changed in 0.1.49

* Enable missing feature by @nesium
* Throttle and coalesce events ([prose-app-web#37](https://github.com/prose-im/prose-app-web/issues/37)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.48...0.1.49)


## What's Changed in 0.1.48

* Add jid to PresenceSubRequest ([#45](https://github.com/prose-im/prose-core-client/issues/45)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.47...0.1.48)


## What's Changed in 0.1.47

* Add some more documentation for new methods ([#45](https://github.com/prose-im/prose-core-client/issues/45)) by @nesium
* Add some documentation for new methods ([#45](https://github.com/prose-im/prose-core-client/issues/45)) by @nesium
* Add contact management methods ([#45](https://github.com/prose-im/prose-core-client/issues/45)) by @nesium
* Use roster element from xmpp_parsers by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.46...0.1.47)


## What's Changed in 0.1.46

* Add method to remove contact from roster ([#45](https://github.com/prose-im/prose-core-client/issues/45)) by @nesium
* Do not throw error when trying to join the same room twice ([#46](https://github.com/prose-im/prose-core-client/issues/46)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.45...0.1.46)


## What's Changed in 0.1.45

* Return error from connect explaining what went wrong ([prose-app-web#16](https://github.com/prose-im/prose-app-web/issues/16)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.44...0.1.45)


## What's Changed in 0.1.44

* Add method to find public channel by name ([#43](https://github.com/prose-im/prose-core-client/issues/43)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.43...0.1.44)


## What's Changed in 0.1.43

* Send messagesUpdated event instead of messagesAppended if message already existed (([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27))) by @nesium
* Parse sent message ‘from’ into a ParticipantId::User ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.42...0.1.43)


## What's Changed in 0.1.42

* Use ParticipantId in Message, MessageSender and Reaction ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.41...0.1.42)


## What's Changed in 0.1.41

* Make MessageSender.id optional, remove (JS) Message.from ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27)) by @nesium
* Embed real jid in MessageSender for received messages ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27)) by @nesium
* Fallback to formatted JID instead of “<anonymous>” for MUC participants by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.40...0.1.41)


## What's Changed in 0.1.40

* Add missing callbacks to delegate interface by @nesium
* Add AccountInfo and AccountInfoChanged event ([prose-app-web#18](https://github.com/prose-im/prose-app-web/issues/18)) by @nesium
* Ensure that logging is only initialized once by @nesium
* Prune hidden-from-sidebar bookmarks when the corresponding room generates a permanent error by @nesium
* Filter hidden sidebar items by @nesium
* Connect to rooms with global/restored availability ([#40](https://github.com/prose-im/prose-core-client/issues/40)) by @nesium
* Send ParticipantsChanged event when occupant availability changes ([#40](https://github.com/prose-im/prose-core-client/issues/40)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.39...0.1.40)


## What's Changed in 0.1.39

* Send presence to all connected rooms ([#40](https://github.com/prose-im/prose-core-client/issues/40)) by @nesium
* When connecting, do not insert placeholders for rooms that are not in the sidebar by @nesium
* Allow logging to be configured by @nesium
* Replace hard-coded date with timestamp ([#41](https://github.com/prose-im/prose-core-client/issues/41)) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.38...0.1.39)


## What's Changed in 0.1.38

* When destroying a room remove it from the sidebar and delete its bookmark by @nesium
* Mark room as disconnected if an error occurred ([prose-app-web#28](https://github.com/prose-im/prose-app-web/issues/28)) by @nesium
* Add temporary workaround for #39 by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.37...0.1.38)


## What's Changed in 0.1.37

* Rename RoomInternals to Room and move Arc inside Room by @nesium
* Improve user experience when populating sidebar initially by @nesium
* Introduce RoomSidebarState on RoomInternals by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.36...0.1.37)


## What's Changed in 0.1.36

* Limit the length of a MUC nickname to 20 chars by @nesium
* Ignore errors when joining a DM chat by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.35...0.1.36)


## What's Changed in 0.1.35

* Return roster even if we’re unable to load details about the contacts by @nesium
* Log version on startup by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.34...0.1.35)


## What's Changed in 0.1.34

* rollback release badge, since there are no releases there by @valeriansaliou
* add github release badge by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.33...0.1.34)


## What's Changed in 0.1.33

* Set has_draft in SidebarItem by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.32...0.1.33)


## What's Changed in 0.1.31

* Move room configuration deeper into infrastructure layer by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.30...0.1.31)


## What's Changed in 0.1.30

* Keep sidebar items in sync with remote changes by @nesium
* Do not send unavailable presence when removing DM from sidebar by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.29...0.1.30)


## What's Changed in 0.1.29

* Add contact to sidebar after receiving a message by @nesium
* Return RoomId of created room instead of RoomEnvelope by @nesium
* Support renaming channels by @nesium
* Add contacts to sidebar when receiving a message by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.28...0.1.29)


## What's Changed in 0.1.28

* Implement sidebar logic by @nesium
* specify when a xep is only partially-supported in the doap file by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.27...0.1.28)


## What's Changed in 0.1.27

* Use BareJid again in MessageDTO by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.26...0.1.27)


## What's Changed in 0.1.26

* add npm publish provenance by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.25...0.1.26)


## What's Changed in 0.1.25

* Add name to various objects return from Room by @nesium
* Return ComposingUser instead of BareJid from load_composing_users method by @nesium
* Treat Forbidden errors as ItemNotFound by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.24...0.1.25)


## What's Changed in 0.1.24

* Look up real jids when loading messages from a muc room by @nesium
* Handle chat states properly in direct message and muc rooms by @nesium
* Treat Message as a thin layer over xmpp-parsers’ Message by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.23...0.1.24)


## What's Changed in 0.1.23

* Only allow DateTime<Utc> as DB keys by @nesium
* Add repository and entity trait + macro by @nesium
* Support DateTime properly as store key, support JID types by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.22...0.1.23)


## What's Changed in 0.1.22

* Add store abstraction over IndexedDB and SQLite by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.21...0.1.22)


## What's Changed in 0.1.21

* Add MUC module (XEP-0045) by @nesium
* Support legacy bookmarks (XEP-0048) by @nesium
* Do not call transformer when future failed by @nesium
* Simplify JID handling by @nesium
* Add bookmark module (XEP-0402) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.20...0.1.21)


## What's Changed in 0.1.20

* broken caps hash calculation by @valeriansaliou
* do not announce xml:lang in caps by @valeriansaliou
* make capabilities debuggable by @valeriansaliou
* Move PresenceMap into util and make it available in crate only by @nesium
* tests by @valeriansaliou
* Keep track of user presences and resolve BareJids to FullJids internally by @nesium
* add all caps disco info by @valeriansaliou
* add npm badge by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.19...0.1.20)


## What's Changed in 0.1.19

* Send empty `before` element if neither before nor after are set by @nesium
* roster favorite group not being picked up due to lowercased key by @valeriansaliou
* bug where other group would always get picked up since contacts had no group set by @valeriansaliou
* Deal with missing message IDs by @nesium
* prose-sdk-js cargo.toml output to package.json by @valeriansaliou
* Send caps when availability is changed by @nesium
* Allow sending caps with presence by @nesium
* sync avatar cache sizes/quality w/ web app saving parameters by @valeriansaliou
* normalize prose website url (caps node) by @valeriansaliou
* Response to last activity request by @nesium
* Respond properly to disco query by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.18...0.1.19)


## What's Changed in 0.1.18

* Allow configuring software version via ProseClientConfig by @nesium
* Support XEP-0092: Software Version by @nesium
* Let client decide/send entity time response by @nesium
* Use DateTime<FixedOffset> for TimeProvider by @nesium
* Handle ping requests by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.17...0.1.18)


## What's Changed in 0.1.17

* Insert current user as contact into cache by @nesium
* Assign each contact to one of a set of predefined groups by @nesium
* Fallback to name of roster item when no name is available in vCard by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.16...0.1.17)


## What's Changed in 0.1.16

* speed up build workflow by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.15...0.1.16)


## What's Changed in 0.1.14

* naming by @valeriansaliou
* add build & release action by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.13...0.1.14)


## What's Changed in 0.1.13

* add release to npm by @valeriansaliou
* clear Rust incompatible scope in wasm-pack package name by @valeriansaliou
* Add readme by @nesium
* Move remaining types from prose-domain into prose-core-client by @nesium
* Move ConnectionProvider back to web app after failed attempt to bundle Strophe.JS by @nesium
* Do not access code property of a DomException which might not contain a numeric value and throw by @nesium
* Fix connection config by @nesium
* Add config object for client by @nesium
* Make sure that events are sent on connect and disconnect by @nesium
* Pass client as first argument in delegate calls by @nesium
* Depend on StropheJS and integrate StropheJSConnection by @nesium
* Support setting availability by @nesium
* Let client only deal with BareJids by @nesium
* Remove status from connect and set_availability methods by @nesium
* Handle presence update properly by @nesium
* Provide handle_pubsub_message default implementation by @nesium
* Make it more clear which module events originate from by @nesium
* Support user_activity in SQLiteCache by @nesium
* Race condition where “now” could be less than the timestamp of a pending future since it was measured before aquiring the lock by @nesium
* Introduce UserMetadata by @nesium
* Support loading last user activity by @nesium
* Support loading entity time and responding to entity time requests by @nesium
* Use chrono instead of SystemTime as TimeProvider by @nesium
* Handle errors thrown by the JS connection by @nesium
* Remove tel: from phone uris by @nesium
* Send ping and timeout events from StropheJS connection by @nesium
* Send ping again by @nesium
* Add method to disconnect and delete cached data in wasm binding by @nesium
* Support first_name, last_name & role in vCard by @nesium
* Simplify code by @nesium
* Save Wasm avatar and profile by @nesium
* Split client_user to better match mods structure by @nesium
* Support user activity by @nesium
* Improve TS type-safety by @nesium
* Add toggleReactionToMessage method to wasm binding by @nesium
* Add “store” element to reaction message by @nesium
* Allow caching the same message twice by @nesium
* Handle _all_ messages by @nesium
* Ignore presence stanzas without type by @nesium
* Add missing functionality in IndexedDBDataCache (mostly) by @nesium
* Save draft by @nesium
* IndexedDB cache passes two more tests by @nesium
* Set up wasm-pack tests by @nesium
* Adapt message to fit into the web app’s shape by @nesium
* Accidentally consuming params that lead to usage of moved values later on by @nesium
* Use strongly typed Jid(s) in Wasm binding by @nesium
* Optimize wasm build by @nesium
* Store and fetch contacts from IndexedDB by @nesium
* Send disco info response again by @nesium
* Make data cache async to support IndexedDB by @nesium
* rename crates by @nesium
* Migrate to xmpp-rs by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.12...0.1.13)


## What's Changed in 0.1.12

* Introduce XMPPElement to hand to modules by @nesium
* Call connection_handler again by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.13.0...0.1.12)


## What's Changed in 0.13.0

* Handle missing vCard and avatar metadata gracefully by @nesium
* Improve parsing of XMPP errors by @nesium
* Prevent failure by converting image to rgb8 before saving by @nesium
* Make avatar metadata dimensions optional by @nesium
* Move ClientBuilder to separate file by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.12.0...0.13.0)


## What's Changed in 0.12.0

* FFI conversion of URLs by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.11.0...0.12.0)


## What's Changed in 0.11.0

* Reorder initial sequence so that caps don’t override availability by @nesium
* Allow specifying availability and status when connecting by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.10.0...0.11.0)


## What's Changed in 0.10.0

* Allow deleting current user profile by @nesium
* Return optional vCard instead of default value by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.9.0...0.10.0)


## What's Changed in 0.9.0

* Support sending presence and persisting it in new AccountSettings by @nesium
* Split client into multiple files by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.8.0...0.9.0)


## What's Changed in 0.8.0

* Add CachePolicy so that clients can load data from cache without hitting the server by @nesium
* Show contacts as unavailable if we didn’t receive a presence yet by @nesium
* Allow saving a draft message per conversation by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.7.0...0.8.0)


## What's Changed in 0.7.0

* Add support for ChatStates by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.6.0...0.7.0)


## What's Changed in 0.5.0

* Split library, integrate most business logic former implemented in Swift by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.4.3...0.5.0)


## What's Changed in 0.4.3

* Add mandatory id to iq stanza by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.4.2...0.4.3)


## What's Changed in 0.4.2

* Subscribe to avatar:metadata not avatar:data pubsub by @nesium
* Send image sha1 to observer after setting avatar by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.4.1...0.4.2)


## What's Changed in 0.4.0

* Add support for setting and loading avatar images by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.3.0...0.4.0)


## What's Changed in 0.3.0

* Load last page if beforeId is not set by @nesium
* Handle empty result sets by @nesium
* Ignore possibly forged carbon messages (CVE-2017-5589) by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.2.0...0.3.0)


## What's Changed in 0.2.0

* Support message carbons by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.9...0.2.0)


## What's Changed in 0.1.9

* Add messageId to reactions by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.8...0.1.9)


## What's Changed in 0.1.7

* Support retracting messages by @nesium
* Support sending message reactions by @nesium
* Use camel-case for callback interface which would otherwise not compile by @nesium
* Add MAM support by @nesium
* Deserialize ‘delay’ element by @nesium
* Load and set MAM archiving preferences by @nesium
* Add methods to modify roster and presence subscriptions by @nesium
* Inject IDProvider instead of referencing UUID by @nesium
* Convert method names to snake case by @nesium
* XMPP extensions & testability by @nesium in [#6](https://github.com/prose-im/prose-core-client/pull/6)

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.5...0.1.7)


## What's Changed in 0.1.5

* Add support for sending presence, chat state and updating messages by @nesium
* Provide JID parsing methods, save JID in client and throw error on connect instead of crashing by @nesium
* Use const strings for namespaces instead of enum by @nesium
* Parse message corrections by @nesium
* Forward received presence stanzas to observer by @nesium
* Add support for chat states by @nesium
* Use strum to serialize/deserialize enums by @nesium
* Add method to send raw XML payloads by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.4...0.1.5)


## What's Changed in 0.1.4

* Support roster items without a group by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.3...0.1.4)


## What's Changed in 0.1.3

* Extend message attributes by @nesium
* Prepare presence support by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.2...0.1.3)


## What's Changed in 0.1.2

* Send connection events to observer by @nesium in [#5](https://github.com/prose-im/prose-core-client/pull/5)
* rename workflow step by @valeriansaliou
* new badge url format by @valeriansaliou
* add readme license to copyright by @valeriansaliou
* badge paths by @valeriansaliou
* normalize README by @valeriansaliou
* mis-attributed comments by @valeriansaliou

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.1...0.1.2)


## What's Changed in 0.1.1

* Request and receive roster by @nesium
* First version that can send and receive messages by @nesium

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.0...0.1.1)


## What's Changed in 0.1.0

* Move global FFI functions into Client struct by @nesium
* Convert project to a workspace by @nesium
* Add basic FFI interface using UniFFI by @nesium
* declare lifetimes when passing client reference around by @valeriansaliou


