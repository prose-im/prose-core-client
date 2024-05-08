use crate::tests::{collections, platform_driver, store, Person};
use anyhow::Result;
use chrono::NaiveDate;
use insta::assert_snapshot;
use prose_store::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_write_transactions_succeed() -> Result<()> {
    let store = store().await?;
    let store1 = store.clone();
    let store2 = store1.clone();

    let semaphore = Arc::new(Semaphore::new(2));
    let semaphore1 = semaphore.clone();
    let semaphore2 = semaphore.clone();

    let transactions_complete = Arc::new(Mutex::new(0));
    let transactions_complete1 = transactions_complete.clone();
    let transactions_complete2 = transactions_complete.clone();

    tokio::spawn(async move {
        let permit = semaphore1.acquire().await.unwrap();
        assert_eq!(semaphore1.available_permits(), 1);

        let tx1 = store1
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await
            .unwrap();
        let people = tx1.writeable_collection(collections::PERSON).unwrap();
        people.set("id-1", &Person::jane_doe()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        assert_eq!(semaphore1.available_permits(), 0);
        assert_eq!(*transactions_complete1.lock().unwrap(), 0);

        tx1.commit().await.unwrap();

        *transactions_complete1.lock().unwrap() += 1;

        drop(permit)
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;

        let permit = semaphore2.acquire().await.unwrap();
        assert_eq!(semaphore2.available_permits(), 0);

        let tx2 = store2
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await
            .unwrap();
        let collection2 = tx2.writeable_collection(collections::PERSON).unwrap();
        collection2.set("id-2", &Person::john_doe()).await.unwrap();
        tx2.commit().await.unwrap();

        *transactions_complete2.lock().unwrap() += 1;

        drop(permit)
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let permit = semaphore.acquire_many(2).await?;
    assert_eq!(*transactions_complete.lock().unwrap(), 2);

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;

    let collection = tx.readable_collection(collections::PERSON)?;

    assert_eq!(collection.get("id-1").await?, Some(Person::jane_doe()));
    assert_eq!(collection.get("id-2").await?, Some(Person::john_doe()));

    drop(permit);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_table_structure() -> Result<()> {
    let store = store().await?;
    let sql = store.describe_table(collections::PERSON).await?;
    assert_snapshot!(sql);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_uses_index() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;
    let birthdays = people.index(&[collections::person::BIRTHDAY])?;

    let sql = birthdays.explain_query_plan(Query::from_range(
        NaiveDate::from_ymd_opt(2020, 01, 02).unwrap()
            ..=NaiveDate::from_ymd_opt(2020, 01, 04).unwrap(),
    ))?;

    assert_snapshot!(sql);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_uses_multicolumn_index() -> Result<()> {
    let driver = platform_driver("multi-column-index");
    let store = Store::open(driver, 1, |event| {
        let tx = &event.tx;

        let collection = tx.create_collection("device_record")?;
        collection.add_index(
            IndexSpec::builder()
                .add_column("account")
                .add_column("user_id")
                .build(),
        )?;

        Ok(())
    })
    .await?;

    let tx = store.transaction_for_reading(&["device_record"]).await?;
    let records = tx.readable_collection("device_record")?;
    let idx = records.index(&["account", "user_id"])?;

    let sql = idx.explain_query_plan(Query::Only(("b@prose.org", 1)))?;

    assert_snapshot!(sql);

    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DeviceRecord {
    account: String,
    user_id: u32,
    id: u32,
    name: String,
}
