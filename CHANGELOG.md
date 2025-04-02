## What's Changed in 0.1.103

* Support vCard kind
* Remove import that slipped in
* Handle workspace info & icon

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.102...0.1.103)


## What's Changed in 0.1.102

* Store unknown properties as-is in vCard4

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.101...0.1.102)


## What's Changed in 0.1.101

* Specify transient dependency explicity and enable js feature to fix wasm build
* Prevent doubly escaped HTML entities in Message Styling text

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.100...0.1.101)


## What's Changed in 0.1.100

* Specify transient dependency explicity and enable js feature to fix wasm build
* Support changing password
* Issue when joining room

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.99...0.1.100)


## What's Changed in 0.1.99

* Let API clients only deal with MessageIds
* Simplify resolving message IDs by returning an id triple
* Remove unused dependencies

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.98...0.1.99)


## What's Changed in 0.1.98

* Added method to preview Markdown

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.97...0.1.98)


## What's Changed in 0.1.97

* Parse message replies XEP-0461
* Add method to send message to MUC that returns the echoed message
* Log panics to a separate method in JSLogger

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.96...0.1.97)


## What's Changed in 0.1.96

* Add avatar to MessageSender
* Ignore MUC owners/admins/members without a node in their JID
* Load avatar images in non-anonymous rooms

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.95...0.1.96)


## What's Changed in 0.1.95

* Replace linebreaks with <br/> in HTML representation of non-Markdown message
* Keep room settings for MUC rooms too in case loading fails after reconnect
* Under certain circumstances the unread/mention count of a Direct Message was reset after reconnecting

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.94...0.1.95)


## What's Changed in 0.1.94

* Expose avatar on AccountInfo and Contact

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.93...0.1.94)


## What's Changed in 0.1.93

* Add avatar to Contact and AccountInfo
* Attachments of sent messages were not stored (and thus not displayed)

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.92...0.1.93)


## What's Changed in 0.1.92

* Load vCard4 properly from PubSub
* Send Markdown content via XEP-0481 and convert body text to XEP-0393: Message Styling

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.91...0.1.92)


## What's Changed in 0.1.91

* Prevent panic when reconnecting rooms after disconnect/reconnect cycle
* Use pretty nicknames in MUCs
* Include optional full name in Contact and UserPresenceInfo
* Publish nickname explicitly when saving profile
* Add id to Avatar
* Use proper key for AvatarRecord
* Refactor user info, nickname and avatar handling
* Add method to load vcard-temp
* Prevent roster from being loaded twice
* Include nickname in MUC presence
* Distinguish between presence and vCard name for participants
* Add client and caps on Participant
* Introduce Avatar struct to support vCard and PEP avatars

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.89...0.1.91)


## What's Changed in 0.1.89

* Inline code from `console_error_panic_hook` crate to log panics via the `tracing` crate

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.88...0.1.89)


## What's Changed in 0.1.88

* Add vcard-temp parser

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.87...0.1.88)


## What's Changed in 0.1.87

* Introduce invisible availability
* Sent messages in MUC rooms would be count as unread

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.86...0.1.87)


## What's Changed in 0.1.86

* Dispatch disconnect events immediately

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.84...0.1.86)


## What's Changed in 0.1.84

* Do not try to access account after logging out

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.83...0.1.84)


## What's Changed in 0.1.83

* Round message timestamps to second precision
* Save received message for newly created rooms

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.82...0.1.83)


## What's Changed in 0.1.82

* Reset state, connect & catchup rooms + send MessagesNeedReload event after reconnect
* Do not count sent messages as unread in sidebar

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.81...0.1.82)


## What's Changed in 0.1.81

* Provide method to set last read message
* Ping MUC rooms regularly and reconnect if needed (XEP-0410)
* Segregate data by account in repositories

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.80...0.1.81)


## What's Changed in 0.1.80

* Channels were not loaded in the web version due to an exception during the connection process

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.79...0.1.80)


## What's Changed in 0.1.79

* Defer deletion of used PreKeys until after catch-up
* Await unused future
* Save MessageId instead of StanzaId for last unread message to allow marking sent messages as read
* Do not show invalid unread count while room is still pending/connecting
* Determine server time offset when connecting to improve accuracy of local timestamps

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.78...0.1.79)


## What's Changed in 0.1.78

* Immediately cancel pending futures upon receiving a Disconnected event
* Defer processing of offline messages until after complete connection
* Improve unread handling and synchronize between clients

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.77...0.1.78)


## What's Changed in 0.1.76

* Add MessageArchiveDomainService to catch up on conversations
* Support full range of MAM queries
* Record timestamp when connection was established
* Determine supported MAM version
* Add local room settings
* Include table name in generated index name
* Add method to get last received message
* Add method to fold rows

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.75...0.1.76)


## What's Changed in 0.1.75

* Delete only messages that belong to a given account
* Add delete_all_in_index method
* Segregate messages in cache by account and room id
* Serialize RoomId identically when used as key or property
* Implement KeyType for every &T: KeyType
* Support multi-column indexes
* Support OMEMO in MUC rooms
* Improve event dispatching for sent/received messages
* Allow empty message bodies for attachment-only messages
* Include mentions in message corrections
* Do not send events for changes to our own compose state in MUC rooms

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.74...0.1.75)


## What's Changed in 0.1.74

* Encrypt message updates
* Decrypt messages in MUC rooms
* Prevent continually trying to load a user’s vCard if none is available
* Pass room_id to SidebarDomainService to prevent creating DM sidebar items when receiving a message in a non-anonymous room
* Sort imports
* Distinguish between NoDevices and NoTrustedDevices errors
* Try to repair session when decryption fails
* Try to unpublish broken devices only once
* Unpublish device if session cannot be started
* Complete session automatically after receiving prekey message

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.73...0.1.74)


## What's Changed in 0.1.73

* Mark sessions as active/inactive when the corresponding devices disappear/reappear
* Start OMEMO sessions for own devices lazily
* Remove UserDeviceKey::min & max which would lead to incorrect queries
* Throw EncryptionError when recipient has no OMEMO devices
* Start OMEMO session on demand
* Introduce EncryptionError and throw when recipient has no OMEMO devices

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.72...0.1.73)


## What's Changed in 0.1.72

* Re-publish own device list if current device is not included in PubSub message
* Handle empty OMEMO messages

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.71...0.1.72)


## What's Changed in 0.1.71

* Ignore failures when querying server components on startup

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.70...0.1.71)


## What's Changed in 0.1.70

* Add interface and support for custom JS logger
* Trim vCard values which would lead to empty user names
* Resolve reaction senders’ names
* Include local timezone in entity time request ([#71](https://github.com/prose-im/prose-core-client/issues/71))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.69...0.1.70)


## What's Changed in 0.1.69

* Cache device list, include trust and fingerprint in DeviceDTO
* Clean OMEMO related data when clearing cache
* Support deleting OMEMO devices

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.68...0.1.69)


## What's Changed in 0.1.67

* Support OMEMO in 1:1 conversations ([#70](https://github.com/prose-im/prose-core-client/issues/70))
* Support numeric keys in get_all methods
* Immediately insert participant in pending direct message rooms so that it can be accessed in API consumers
* Prevent MUC rooms from sending message history

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.66...0.1.67)


## What's Changed in 0.1.66

* Parse private MUC messages and mark them as transient

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.65...0.1.66)


## What's Changed in 0.1.65

* Prevent dangling futures when client is disconnected
* Update sent reactions in a MUC room
* Allow providing a media type when requesting an UploadSlot
* Interpret m4a extension as audio/mp4

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.64...0.1.65)


## What's Changed in 0.1.64

* Support mentions
* Target messages for chat markers via StanzaId in MUC rooms ([#60](https://github.com/prose-im/prose-core-client/issues/60))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.63...0.1.64)


## What's Changed in 0.1.63

* Treat MUC messages sent by us correctly as “sent message”
* Improve carbon handling

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.62...0.1.63)


## What's Changed in 0.1.62

* Allow message payloads to target messages by their stanza id ([#60](https://github.com/prose-im/prose-core-client/issues/60))
* Remove assertion that leads to crash when receiving a cached/delayed message on login

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.61...0.1.62)


## What's Changed in 0.1.61

* Do not consume MAM messages in future to join room ([#55](https://github.com/prose-im/prose-core-client/issues/55))
* Increase unread count in channels ([#49](https://github.com/prose-im/prose-core-client/issues/49))
* Update unread count in sidebar ([#49](https://github.com/prose-im/prose-core-client/issues/49))
* Reduce newer cached messages into older messages loaded from server ([#59](https://github.com/prose-im/prose-core-client/issues/59))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.60...0.1.61)


## What's Changed in 0.1.59

* Fill MessageResultSet to a minimum (or more) of guaranteed messages ([#58](https://github.com/prose-im/prose-core-client/issues/58))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.58...0.1.59)


## What's Changed in 0.1.58

* Support loading older messages from MAM ([#58](https://github.com/prose-im/prose-core-client/issues/58))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.57...0.1.58)


## What's Changed in 0.1.57

* Rejoining an already joined group would not bring it back into the sidebar ([#56](https://github.com/prose-im/prose-core-client/issues/56))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.56...0.1.57)


## What's Changed in 0.1.56

* Regression where room topic was not available anymore introduced in #52
* Regression in merging/lookup of participants introduced in #52 ([#54](https://github.com/prose-im/prose-core-client/issues/54))
* Prevent stanzas being handled out of order with the native connector

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.55...0.1.56)


## What's Changed in 0.1.55

* Show current user in participants list when affiliation is less than member ([#53](https://github.com/prose-im/prose-core-client/issues/53))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.54...0.1.55)


## What's Changed in 0.1.54

* Thumbnails set via Attachment::video_attachment were discarded

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.53...0.1.54)


## What's Changed in 0.1.53

* Support XEP-0385 attachments and thumbnails ([#24](https://github.com/prose-im/prose-core-client/issues/24))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.52...0.1.53)


## What's Changed in 0.1.52

* Do not dispatch event when relaying a message
* Dispatch event again when sending a message ([#24](https://github.com/prose-im/prose-core-client/issues/24))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.51...0.1.52)


## What's Changed in 0.1.51

* Add some documentation to wasm bindings ([#24](https://github.com/prose-im/prose-core-client/issues/24))
* Support sending/receiving messages with attachments ([#24](https://github.com/prose-im/prose-core-client/issues/24))
* Add functionality to request upload slot ([#24](https://github.com/prose-im/prose-core-client/issues/24))
* Parse HTTP upload endpoint into server features ([#24](https://github.com/prose-im/prose-core-client/issues/24))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.50...0.1.51)


## What's Changed in 0.1.50

* Improve error handling ([prose-app-web#38](https://github.com/prose-im/prose-app-web/issues/38))
* Introduce MessageMetadata with isRead flag ([#48](https://github.com/prose-im/prose-core-client/issues/48))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.49...0.1.50)


## What's Changed in 0.1.49

* Enable missing feature
* Throttle and coalesce events ([prose-app-web#37](https://github.com/prose-im/prose-app-web/issues/37))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.48...0.1.49)


## What's Changed in 0.1.48

* Add jid to PresenceSubRequest ([#45](https://github.com/prose-im/prose-core-client/issues/45))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.47...0.1.48)


## What's Changed in 0.1.47

* Add some more documentation for new methods ([#45](https://github.com/prose-im/prose-core-client/issues/45))
* Add some documentation for new methods ([#45](https://github.com/prose-im/prose-core-client/issues/45))
* Add contact management methods ([#45](https://github.com/prose-im/prose-core-client/issues/45))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.46...0.1.47)


## What's Changed in 0.1.46

* Add method to remove contact from roster ([#45](https://github.com/prose-im/prose-core-client/issues/45))
* Do not throw error when trying to join the same room twice ([#46](https://github.com/prose-im/prose-core-client/issues/46))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.45...0.1.46)


## What's Changed in 0.1.45

* Return error from connect explaining what went wrong ([prose-app-web#16](https://github.com/prose-im/prose-app-web/issues/16))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.44...0.1.45)


## What's Changed in 0.1.44

* Add method to find public channel by name ([#43](https://github.com/prose-im/prose-core-client/issues/43))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.43...0.1.44)


## What's Changed in 0.1.43

* Send messagesUpdated event instead of messagesAppended if message already existed (([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27)))
* Parse sent message ‘from’ into a ParticipantId::User ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.42...0.1.43)


## What's Changed in 0.1.42

* Use ParticipantId in Message, MessageSender and Reaction ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.41...0.1.42)


## What's Changed in 0.1.41

* Embed real jid in MessageSender for received messages ([prose-app-web#27](https://github.com/prose-im/prose-app-web/issues/27))
* Fallback to formatted JID instead of “<anonymous>” for MUC participants

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.40...0.1.41)


## What's Changed in 0.1.40

* Add missing callbacks to delegate interface
* Add AccountInfo and AccountInfoChanged event ([prose-app-web#18](https://github.com/prose-im/prose-app-web/issues/18))
* Ensure that logging is only initialized once
* Prune hidden-from-sidebar bookmarks when the corresponding room generates a permanent error
* Filter hidden sidebar items
* Connect to rooms with global/restored availability ([#40](https://github.com/prose-im/prose-core-client/issues/40))
* Send ParticipantsChanged event when occupant availability changes ([#40](https://github.com/prose-im/prose-core-client/issues/40))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.39...0.1.40)


## What's Changed in 0.1.39

* Send presence to all connected rooms ([#40](https://github.com/prose-im/prose-core-client/issues/40))
* When connecting, do not insert placeholders for rooms that are not in the sidebar
* Allow logging to be configured
* Replace hard-coded date with timestamp ([#41](https://github.com/prose-im/prose-core-client/issues/41))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.38...0.1.39)


## What's Changed in 0.1.38

* When destroying a room remove it from the sidebar and delete its bookmark
* Mark room as disconnected if an error occurred ([prose-app-web#28](https://github.com/prose-im/prose-app-web/issues/28))
* Add temporary workaround for #39

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.37...0.1.38)


## What's Changed in 0.1.37

* Improve user experience when populating sidebar initially

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.36...0.1.37)


## What's Changed in 0.1.36

* Limit the length of a MUC nickname to 20 chars
* Ignore errors when joining a DM chat

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.35...0.1.36)


## What's Changed in 0.1.35

* Return roster even if we’re unable to load details about the contacts
* Log version on startup

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.34...0.1.35)


## What's Changed in 0.1.34

* rollback release badge, since there are no releases there
* add github release badge

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.33...0.1.34)


## What's Changed in 0.1.33

* Set has_draft in SidebarItem

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.32...0.1.33)


## What's Changed in 0.1.30

* Keep sidebar items in sync with remote changes
* Do not send unavailable presence when removing DM from sidebar

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.29...0.1.30)


## What's Changed in 0.1.29

* Add contact to sidebar after receiving a message
* Support renaming channels
* Add contacts to sidebar when receiving a message

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.28...0.1.29)


## What's Changed in 0.1.28

* Implement sidebar logic
* specify when a xep is only partially-supported in the doap file

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.27...0.1.28)


## What's Changed in 0.1.27

* Use BareJid again in MessageDTO

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.26...0.1.27)


## What's Changed in 0.1.26

* add npm publish provenance

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.25...0.1.26)


## What's Changed in 0.1.25

* Add name to various objects return from Room
* Return ComposingUser instead of BareJid from load_composing_users method
* Treat Forbidden errors as ItemNotFound

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.24...0.1.25)


## What's Changed in 0.1.24

* Look up real jids when loading messages from a muc room
* Handle chat states properly in direct message and muc rooms

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.23...0.1.24)


## What's Changed in 0.1.23

* Only allow DateTime<Utc> as DB keys
* Add repository and entity trait + macro
* Support DateTime properly as store key, support JID types

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.22...0.1.23)


## What's Changed in 0.1.22

* Add store abstraction over IndexedDB and SQLite

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.21...0.1.22)


## What's Changed in 0.1.21

* Add MUC module (XEP-0045)
* Support legacy bookmarks (XEP-0048)
* Do not call transformer when future failed
* Add bookmark module (XEP-0402)

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.20...0.1.21)


## What's Changed in 0.1.20

* broken caps hash calculation
* do not announce xml:lang in caps
* make capabilities debuggable
* tests
* Keep track of user presences and resolve BareJids to FullJids internally
* add all caps disco info
* add npm badge

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.19...0.1.20)


## What's Changed in 0.1.19

* Send empty `before` element if neither before nor after are set
* roster favorite group not being picked up due to lowercased key
* bug where other group would always get picked up since contacts had no group set
* Deal with missing message IDs
* prose-sdk-js cargo.toml output to package.json
* Send caps when availability is changed
* Allow sending caps with presence
* sync avatar cache sizes/quality w/ web app saving parameters
* normalize prose website url (caps node)
* Response to last activity request
* Respond properly to disco query

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.18...0.1.19)


## What's Changed in 0.1.18

* Allow configuring software version via ProseClientConfig
* Support XEP-0092: Software Version
* Handle ping requests

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.17...0.1.18)


## What's Changed in 0.1.17

* Insert current user as contact into cache
* Assign each contact to one of a set of predefined groups
* Fallback to name of roster item when no name is available in vCard

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.16...0.1.17)


## What's Changed in 0.1.16

* speed up build workflow

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.15...0.1.16)


## What's Changed in 0.1.14

* naming
* add build & release action

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.13...0.1.14)


## What's Changed in 0.1.13

* add release to npm
* clear Rust incompatible scope in wasm-pack package name
* Add readme
* Do not access code property of a DomException which might not contain a numeric value and throw
* Fix connection config
* Add config object for client
* Make sure that events are sent on connect and disconnect
* Support setting availability
* Handle presence update properly
* Support user_activity in SQLiteCache
* Race condition where “now” could be less than the timestamp of a pending future since it was measured before aquiring the lock
* Introduce UserMetadata
* Support loading last user activity
* Support loading entity time and responding to entity time requests
* Handle errors thrown by the JS connection
* Remove tel: from phone uris
* Send ping and timeout events from StropheJS connection
* Send ping again
* Add method to disconnect and delete cached data in wasm binding
* Support first_name, last_name & role in vCard
* Save Wasm avatar and profile
* Support user activity
* Improve TS type-safety
* Add toggleReactionToMessage method to wasm binding
* Add “store” element to reaction message
* Allow caching the same message twice
* Handle _all_ messages
* Ignore presence stanzas without type
* Add missing functionality in IndexedDBDataCache (mostly)
* Save draft
* IndexedDB cache passes two more tests
* Set up wasm-pack tests
* Adapt message to fit into the web app’s shape
* Accidentally consuming params that lead to usage of moved values later on
* Use strongly typed Jid(s) in Wasm binding
* Optimize wasm build
* Store and fetch contacts from IndexedDB
* Send disco info response again

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.12...0.1.13)


## What's Changed in 0.1.12

* Call connection_handler again

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.13.0...0.1.12)


## What's Changed in 0.13.0

* Handle missing vCard and avatar metadata gracefully
* Improve parsing of XMPP errors
* Prevent failure by converting image to rgb8 before saving
* Make avatar metadata dimensions optional

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.12.0...0.13.0)


## What's Changed in 0.12.0

* FFI conversion of URLs

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.11.0...0.12.0)


## What's Changed in 0.11.0

* Reorder initial sequence so that caps don’t override availability
* Allow specifying availability and status when connecting

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.10.0...0.11.0)


## What's Changed in 0.10.0

* Allow deleting current user profile

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.9.0...0.10.0)


## What's Changed in 0.9.0

* Support sending presence and persisting it in new AccountSettings

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.8.0...0.9.0)


## What's Changed in 0.8.0

* Add CachePolicy so that clients can load data from cache without hitting the server
* Show contacts as unavailable if we didn’t receive a presence yet
* Allow saving a draft message per conversation

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.7.0...0.8.0)


## What's Changed in 0.7.0

* Add support for ChatStates

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.6.0...0.7.0)


## What's Changed in 0.4.3

* Add mandatory id to iq stanza

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.4.2...0.4.3)


## What's Changed in 0.4.0

* Add support for setting and loading avatar images

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.3.0...0.4.0)


## What's Changed in 0.3.0

* Load last page if beforeId is not set
* Handle empty result sets
* Ignore possibly forged carbon messages (CVE-2017-5589)

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.2.0...0.3.0)


## What's Changed in 0.2.0

* Support message carbons

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.9...0.2.0)


## What's Changed in 0.1.9

* Add messageId to reactions

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.8...0.1.9)


## What's Changed in 0.1.7

* Support retracting messages
* Support sending message reactions
* Use camel-case for callback interface which would otherwise not compile
* Add MAM support
* Deserialize ‘delay’ element
* Load and set MAM archiving preferences
* Add methods to modify roster and presence subscriptions
* Introduce XMPP extensions & testability ([#6](https://github.com/prose-im/prose-core-client/issues/6))

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.5...0.1.7)


## What's Changed in 0.1.5

* Add support for sending presence, chat state and updating messages
* Provide JID parsing methods, save JID in client and throw error on connect instead of crashing
* Parse message corrections
* Forward received presence stanzas to observer
* Add support for chat states
* Add method to send raw XML payloads

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.4...0.1.5)


## What's Changed in 0.1.4

* Support roster items without a group

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.3...0.1.4)


## What's Changed in 0.1.3

* Extend message attributes
* Prepare presence support

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.2...0.1.3)


## What's Changed in 0.1.2

* Send connection events to observer ([#5](https://github.com/prose-im/prose-core-client/issues/5))
* rename workflow step
* new badge url format
* add readme license to copyright
* badge paths
* normalize README
* mis-attributed comments

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.1...0.1.2)


## What's Changed in 0.1.1

* Request and receive roster
* First version that can send and receive messages

[Full Changelog](https://github.com/prose-im/prose-core-client/compare/0.1.0...0.1.1)


## What's Changed in 0.1.0

* Add basic FFI interface using UniFFI
* declare lifetimes when passing client reference around


