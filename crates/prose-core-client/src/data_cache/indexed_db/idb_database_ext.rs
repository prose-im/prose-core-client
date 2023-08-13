// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::{IdbObjectStore, IdbTransactionMode};
use indexed_db_futures::{IdbDatabase, IdbIndex, IdbQuerySource};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use super::cache::Result;

#[async_trait(? Send)]
pub trait IdbDatabaseExt {
    async fn set_value<T: Serialize + ?Sized>(
        &self,
        store: impl AsRef<str>,
        key: impl AsRef<str>,
        value: &T,
    ) -> Result<()>;

    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        store: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Option<T>>;

    async fn delete_value(&self, store: impl AsRef<str>, key: impl AsRef<str>) -> Result<()>;

    async fn clear_stores(&self, store: &[&str]) -> Result<()>;
}

#[async_trait(? Send)]
impl IdbDatabaseExt for IdbDatabase {
    async fn set_value<T: Serialize + ?Sized>(
        &self,
        store: impl AsRef<str>,
        key: impl AsRef<str>,
        value: &T,
    ) -> Result<()> {
        let tx =
            self.transaction_on_one_with_mode(store.as_ref(), IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store.as_ref())?;
        store.set_value(key, value)?;
        tx.await.into_result()?;
        Ok(())
    }

    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        store: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Option<T>> {
        let tx = self.transaction_on_one_with_mode(store.as_ref(), IdbTransactionMode::Readonly)?;
        let store = tx.object_store(store.as_ref())?;
        store.get_value(key.as_ref()).await
    }

    async fn clear_stores(&self, stores: &[&str]) -> Result<()> {
        let tx = self.transaction_on_multi_with_mode(stores, IdbTransactionMode::Readwrite)?;
        for store in stores {
            let store = tx.object_store(store)?;
            store.clear()?;
        }
        tx.await.into_result()?;
        Ok(())
    }

    async fn delete_value(&self, store: impl AsRef<str>, key: impl AsRef<str>) -> Result<()> {
        let tx =
            self.transaction_on_one_with_mode(store.as_ref(), IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store.as_ref())?;
        store.delete(&JsValue::from_str(key.as_ref()))?;
        tx.await.into_result()?;
        Ok(())
    }
}

#[async_trait(? Send)]
pub trait IdbObjectStoreExtGet {
    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Option<T>>;

    async fn get_all_values<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Vec<T>>;
}

pub trait IdbObjectStoreExtSet {
    fn set_value<T: Serialize + ?Sized>(&self, key: impl AsRef<str>, value: &T) -> Result<()>;
}

#[async_trait(? Send)]
impl IdbObjectStoreExtGet for IdbObjectStore<'_> {
    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Option<T>> {
        let value: Option<T> = self
            .get(&JsValue::from_str(key.as_ref()))?
            .await?
            .map(|value| value.into_serde())
            .transpose()?;
        Ok(value)
    }

    async fn get_all_values<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Vec<T>> {
        let values = self
            .get_all_with_key(&JsValue::from_str(key.as_ref()))?
            .await?
            .into_iter()
            .map(|value| value.into_serde())
            .collect::<Result<Vec<T>, _>>()?;
        Ok(values)
    }
}

impl IdbObjectStoreExtSet for IdbObjectStore<'_> {
    fn set_value<T: Serialize + ?Sized>(&self, key: impl AsRef<str>, value: &T) -> Result<()> {
        self.put_key_val(
            &JsValue::from_str(key.as_ref()),
            &JsValue::from_serde(value)?,
        )?;
        Ok(())
    }
}

#[async_trait(? Send)]
impl IdbObjectStoreExtGet for IdbIndex<'_> {
    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Option<T>> {
        let value: Option<T> = self
            .get(&JsValue::from_str(key.as_ref()))?
            .await?
            .map(|value| value.into_serde())
            .transpose()?;
        Ok(value)
    }

    async fn get_all_values<T: for<'de> Deserialize<'de>>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Vec<T>> {
        let values = self
            .get_all_with_key(&JsValue::from_str(key.as_ref()))?
            .await?
            .into_iter()
            .map(|value| value.into_serde())
            .collect::<Result<Vec<T>, _>>()?;
        Ok(values)
    }
}
