use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::IdbKeyRange;
use jid::BareJid;
use wasm_bindgen::JsValue;

use prose_xmpp::stanza::message;

use crate::data_cache::indexed_db::cache::{keys, IndexedDBDataCacheError};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::MessageCache;
use crate::types::{MessageLike, Page};

use super::cache::Result;

#[async_trait(? Send)]
impl MessageCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;

        for message in messages {
            store.put_key_val(
                &JsValue::from_str(message.id.as_ref()),
                &JsValue::from_serde(message)?,
            )?;
        }

        tx.await.into_result()?;
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        _conversation: &BareJid,
        targets: &[message::Id],
        _newer_than: impl Into<Option<&'a message::Id>>,
        _include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let target_idx = store.index(keys::messages::TARGET_INDEX)?;

        for target in targets {
            let range = IdbKeyRange::only(&JsValue::from_str(target.as_ref()))
                .map_err(|_| IndexedDBDataCacheError::InvalidDBKey)?;
            let _cursor = target_idx.open_key_cursor_with_range_owned(range)?.await?;
        }

        // openRequest.onsuccess = (event) => {
        //     const db = event.target.result;
        //     const messagesStore = db.transaction('messages', 'readonly').objectStore('messages');
        //     const targetIndex = messagesStore.index('target');
        //
        //     const cursorRequest = targetIndex.openCursor(IDBKeyRange.only(searchValue));
        //
        //     cursorRequest.onsuccess = (event) => {
        //         const cursor = event.target.result;
        //         if (cursor) {
        //             // If the "target" key matches the search value, add it to the results array
        //             if (cursor.value.target === searchValue) {
        //                 results.push(cursor.value);
        //             }
        //
        //             // Continue searching
        //             cursor.continue();
        //         } else {
        //             // If the cursor is null, we have processed all records
        //             // Now let's check if the "id" key matches the search value
        //             messagesStore.get(searchValue).onsuccess = (event) => {
        //                 if (event.target.result) {
        //                     // Add the result to the results array if it matches the search value
        //                     results.push(event.target.result);
        //                 }
        //
        //                 console.log(results); // Do something with the fetched records
        //             };
        //         }
        //     };
        // };

        Ok(vec![])
    }

    async fn load_messages_before(
        &self,
        _conversation: &BareJid,
        _older_than: Option<&message::Id>,
        _max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        Ok(None)
    }

    async fn load_messages_after(
        &self,
        _conversation: &BareJid,
        _newer_than: &message::Id,
        _max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    async fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        _message_id: &message::Id,
    ) -> Result<Option<message::stanza_id::Id>> {
        Ok(None)
    }

    async fn save_draft(&self, _conversation: &BareJid, _text: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn load_draft(&self, _conversation: &BareJid) -> Result<Option<String>> {
        Ok(None)
    }
}
