use crate::tests::{collections, platform_driver, store, Person};
use anyhow::Result;
use chrono::NaiveDate;
use insta::assert_snapshot;
use prose_store::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;

/// Tests that multiple tasks attempting write transactions are serialized without deadlocking.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_competing_write_transactions_dont_deadlock() -> Result<()> {
    let store = store().await?;
    let store1 = store.clone();
    let store2 = store.clone();

    let barrier = Arc::new(Barrier::new(2));
    let barrier1 = barrier.clone();
    let barrier2 = barrier.clone();

    let tx1 = tokio::spawn(async move {
        let tx = store1
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await?;
        let people = tx.writeable_collection(collections::PERSON)?;

        barrier1.wait().await;

        // Wait for tx2 trying to start the transaction…
        tokio::time::sleep(Duration::from_millis(200)).await;

        people.set("id-1", &Person::jane_doe()).await?;
        tx.commit().await
    });

    let tx2 = tokio::spawn(async move {
        // Wait for tx1 to start a transaction…
        barrier2.wait().await;

        let tx = store2
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await?;
        let people = tx.writeable_collection(collections::PERSON)?;
        people.set("id-2", &Person::john_doe()).await?;
        tx.commit().await
    });

    tx1.await??;
    tx2.await??;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let collection = tx.readable_collection(collections::PERSON)?;
    assert_eq!(collection.get("id-1").await?, Some(Person::jane_doe()));
    assert_eq!(collection.get("id-2").await?, Some(Person::john_doe()));

    Ok(())
}

/// Stress test to verify that the write lock prevents SQLITE_BUSY errors under heavy concurrent
/// write load.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_writes_never_get_sqlite_busy() -> Result<()> {
    let store = store().await?;

    let num_tasks = 8;
    let writes_per_task = 100;

    let mut handles = vec![];

    for task_idx in 0..num_tasks {
        let store = store.clone();
        let handle = tokio::spawn(async move {
            for write_idx in 0..writes_per_task {
                let tx = store
                    .transaction_for_reading_and_writing(&[collections::PERSON])
                    .await?;
                let people = tx.writeable_collection(collections::PERSON)?;
                people
                    .set(&format!("{task_idx}-{write_idx}"), &Person::jane_doe())
                    .await?;
                // Increase likelihood of overlap…
                tokio::time::sleep(Duration::from_millis(10)).await;
                tx.commit().await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let collection = tx.readable_collection(collections::PERSON)?;

    for task_idx in 0..num_tasks {
        for write_idx in 0..writes_per_task {
            assert!(collection
                .get::<_, Person>(&format!("{task_idx}-{write_idx}"))
                .await?
                .is_some());
        }
    }

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
