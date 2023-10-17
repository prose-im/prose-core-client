// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(not(target_arch = "wasm32"))]
mod sqlite;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use prose_store::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use tokio::test as async_test;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as async_test;

pub mod collections {
    pub const PERSON: &str = "person";
    pub const CAMERA: &str = "camera";
    pub const BOOK: &str = "book";

    pub mod person {
        pub const BIRTHDAY: &str = "birthday";
    }

    pub mod book {
        pub const TITLE: &str = "title";
        pub const PUBLISHED_AT: &str = "published_at";
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Person {
    pub name: String,
    pub birthday: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Camera {
    pub brand: String,
    pub model: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Book {
    pub title: String,
    pub published_at: DateTime<Utc>,
}

impl Person {
    pub fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            birthday: Default::default(),
        }
    }

    pub fn john_doe() -> Self {
        Self {
            name: "John Doe".to_string(),
            birthday: Default::default(),
        }
    }

    pub fn jane_doe() -> Self {
        Self {
            name: "Jane Doe".to_string(),
            birthday: Default::default(),
        }
    }
}

impl Camera {
    pub fn canon_5d() -> Self {
        Self {
            brand: "Canon".to_string(),
            model: "5D".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn store() -> Result<Store<IndexedDBDriver>> {
    open_store(IndexedDBDriver::new("test-db")).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn store() -> Result<Store<SqliteDriver>> {
    let path = tempfile::tempdir().unwrap().path().join("test.sqlite");
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    println!("Opening DB at {:?}", path);
    open_store(SqliteDriver::new(path)).await
}

async fn open_store<T: Driver>(driver: T) -> Result<Store<T>> {
    let store = Store::open(driver, 1, |event| {
        let store = event.tx.create_collection(collections::PERSON)?;
        store.add_index(IndexSpec::builder(collections::person::BIRTHDAY).build())?;

        event.tx.create_collection(collections::CAMERA)?;

        let store = event.tx.create_collection(collections::BOOK)?;
        store.add_index(
            IndexSpec::builder(collections::book::TITLE)
                .unique()
                .build(),
        )?;
        store.add_index(IndexSpec::builder(collections::book::PUBLISHED_AT).build())?;

        let mut names = event.tx.collection_names()?;
        names.sort();

        assert_eq!(
            names,
            vec![
                collections::BOOK.to_string(),
                collections::CAMERA.to_string(),
                collections::PERSON.to_string()
            ]
        );

        Ok(())
    })
    .await?;
    store.truncate_all_collections().await?;
    Ok(store)
}

#[async_test]
async fn test_set_and_get() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;
    let cameras = tx.writeable_collection(collections::CAMERA)?;

    people.set("id-1", &Person::jane_doe()).await?;
    people.set("id-2", &Person::john_doe()).await?;

    cameras.set("id-1", &Camera::canon_5d()).await?;
    cameras.set("id-2", &Camera::canon_5d()).await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;
    let cameras = tx.readable_collection(collections::CAMERA)?;

    assert_eq!(people.get("id-1").await?, Some(Person::jane_doe()));
    assert_eq!(people.get("id-2").await?, Some(Person::john_doe()));
    assert_eq!(people.get::<_, Person>("id-3").await?, None);

    assert_eq!(cameras.get("id-1").await?, Some(Camera::canon_5d()));
    assert_eq!(cameras.get("id-2").await?, Some(Camera::canon_5d()));
    assert_eq!(people.get::<_, Camera>("id-3").await?, None);

    Ok(())
}

#[async_test]
async fn test_begin_transaction_with_invalid_collection() -> Result<()> {
    let store = store().await?;

    let result = store
        .transaction_for_reading_and_writing(&["does-not-exist"])
        .await;
    assert!(result.is_err());

    Ok(())
}

#[async_test]
async fn test_access_invalid_collection_from_transaction() -> Result<()> {
    let store = store().await?;

    {
        let tx = store
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await?;

        let result = tx.writeable_collection("does-not-exist");
        assert!(result.is_err());

        let result = tx.readable_collection("does-not-exist");
        assert!(result.is_err());
    }

    {
        let tx = store
            .transaction_for_reading(&[collections::PERSON])
            .await?;

        let result = tx.readable_collection("does-not-exist");
        assert!(result.is_err());
    }

    Ok(())
}

#[async_test]
async fn test_access_invalid_index() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;

    let result = people.index("does-not-exist");
    assert!(result.is_err());

    Ok(())
}

#[async_test]
async fn test_access_collection_not_included_in_transaction() -> Result<()> {
    let store = store().await?;

    {
        let tx = store
            .transaction_for_reading_and_writing(&[collections::PERSON])
            .await?;

        let result = tx.writeable_collection(collections::CAMERA);
        assert!(result.is_err());

        let result = tx.readable_collection(collections::CAMERA);
        assert!(result.is_err());
    }

    {
        let tx = store
            .transaction_for_reading(&[collections::PERSON])
            .await?;

        let result = tx.readable_collection(collections::CAMERA);
        assert!(result.is_err());
    }

    Ok(())
}

#[async_test]
async fn test_get_from_collection_and_index() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;

    let people = tx.writeable_collection(collections::PERSON)?;

    people
        .set(
            "id-1",
            &Person {
                name: "Amelia".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-2",
            &Person {
                name: "Benjamin".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
            },
        )
        .await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;

    let people = tx.readable_collection(collections::PERSON)?;
    let birthdays = people.index(collections::person::BIRTHDAY)?;

    assert_eq!(
        people.get("id-1").await?,
        Some(Person {
            name: "Amelia".to_string(),
            birthday: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
        })
    );
    assert_eq!(
        birthdays
            .get(&NaiveDate::from_ymd_opt(2020, 01, 02).unwrap())
            .await?,
        Some(Person {
            name: "Benjamin".to_string(),
            birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
        })
    );
    assert_eq!(
        birthdays
            .get_all_values::<Person>(
                Query::Only(NaiveDate::from_ymd_opt(2020, 01, 02).unwrap()),
                QueryDirection::Forward,
                None
            )
            .await?
            .first(),
        Some(&Person {
            name: "Benjamin".to_string(),
            birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
        })
    );

    Ok(())
}

#[async_test]
async fn test_set_conflict() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;

    let people = tx.writeable_collection(collections::PERSON)?;
    people.set("id-1", &Person::jane_doe()).await?;
    assert!(people.set("id-1", &Person::john_doe()).await.is_err());

    Ok(())
}

#[async_test]
async fn test_set_conflict_in_unique_index() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::BOOK])
        .await?;

    let books = tx.writeable_collection(collections::BOOK)?;
    books
        .set(
            "id-1",
            &Book {
                title: "My Book".to_string(),
                published_at: Utc.with_ymd_and_hms(2023, 7, 20, 18, 00, 00).unwrap(),
            },
        )
        .await?;

    let result = books
        .set(
            "id-2",
            &Book {
                title: "My Book".to_string(),
                published_at: Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap(),
            },
        )
        .await;

    // Slightly different behavior between implementations ATM…

    #[cfg(not(target_arch = "wasm32"))]
    {
        assert!(result.is_err());
        return Ok(());
    }

    #[cfg(target_arch = "wasm32")]
    {
        assert!(!result.is_err());
        assert!(tx.commit().await.is_err());
        return Ok(());
    }
}

#[async_test]
async fn test_date_time() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::BOOK])
        .await?;

    let books = tx.writeable_collection(collections::BOOK)?;
    let publish_dates = books.index(collections::book::PUBLISHED_AT)?;

    books.put(
        "id-1",
        &Book {
            title: "Book 1".to_string(),
            published_at: Utc.with_ymd_and_hms(2023, 7, 20, 18, 00, 00).unwrap(),
        },
    )?;
    books.put(
        "id-2",
        &Book {
            title: "Book 2".to_string(),
            published_at: Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap(),
        },
    )?;
    books.put(
        "id-3",
        &Book {
            title: "Book 3".to_string(),
            published_at: Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap(),
        },
    )?;
    books.put(
        "id-4",
        &Book {
            title: "Book 4".to_string(),
            published_at: Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap(),
        },
    )?;

    let value = publish_dates
        .get_all_filtered::<Book, _>(
            Query::Only(Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap()),
            Default::default(),
            None,
            |_, book| Some(book.title),
        )
        .await?;

    assert_eq!(
        value,
        [
            "Book 2".to_string(),
            "Book 3".to_string(),
            "Book 4".to_string()
        ]
    );

    assert_eq!(
        publish_dates
            .get::<_, Book>(&Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap())
            .await?
            .map(|book| book.title),
        Some("Book 2".to_string())
    );
    assert!(
        publish_dates
            .contains_key(&Utc.with_ymd_and_hms(2023, 7, 21, 18, 00, 00).unwrap())
            .await?
    );

    Ok(())
}

#[async_test]
async fn test_put_no_conflict() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people.put("id-1", &Person::jane_doe())?;
    people.put("id-1", &Person::john_doe())?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;

    assert_eq!(people.get("id-1").await?, Some(Person::john_doe()));

    Ok(())
}

#[async_test]
async fn test_contains_key() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;
    let cameras = tx.writeable_collection(collections::CAMERA)?;

    people.set("id-2", &Person::jane_doe()).await?;
    cameras.set("id-1", &Camera::canon_5d()).await?;

    assert!(!people.contains_key("id-1").await?);
    assert!(people.contains_key("id-2").await?);
    assert!(cameras.contains_key("id-1").await?);
    assert!(!cameras.contains_key("id-2").await?);

    Ok(())
}

#[async_test]
async fn test_delete() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people.set("id-1", &Person::jane_doe()).await?;
    people.set("id-2", &Person::john_doe()).await?;

    assert!(people.contains_key("id-1").await?);
    assert!(people.contains_key("id-2").await?);

    people.delete("id-2")?;

    assert!(people.contains_key("id-1").await?);
    assert!(!people.contains_key("id-2").await?);

    tx.commit().await?;

    assert!(store.contains_key(collections::PERSON, "id-1").await?);
    store.delete(collections::PERSON, "id-1").await?;
    assert!(!store.contains_key(collections::PERSON, "id-1").await?);

    Ok(())
}

#[async_test]
async fn test_collection_names() -> Result<()> {
    let store = store().await?;

    let mut names = store.collection_names().await?;
    names.sort();

    assert_eq!(
        names,
        vec![
            collections::BOOK.to_string(),
            collections::CAMERA.to_string(),
            collections::PERSON.to_string()
        ]
    );

    Ok(())
}

#[async_test]
async fn test_truncate() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;
    let cameras = tx.writeable_collection(collections::CAMERA)?;

    people.set("id-1", &Person::jane_doe()).await?;
    people.set("id-2", &Person::john_doe()).await?;
    cameras.set("id-1", &Camera::canon_5d()).await?;

    assert!(people.contains_key("id-1").await?);
    assert!(people.contains_key("id-2").await?);
    assert!(cameras.contains_key("id-1").await?);

    people.truncate()?;

    assert!(!people.contains_key("id-1").await?);
    assert!(!people.contains_key("id-2").await?);
    assert!(cameras.contains_key("id-1").await?);

    Ok(())
}

#[async_test]
async fn test_truncate_all() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;
    let cameras = tx.writeable_collection(collections::CAMERA)?;

    people.set("id-1", &Person::jane_doe()).await?;
    people.set("id-2", &Person::john_doe()).await?;
    cameras.set("id-1", &Camera::canon_5d()).await?;

    assert!(people.contains_key("id-1").await?);
    assert!(people.contains_key("id-2").await?);
    assert!(cameras.contains_key("id-1").await?);

    tx.commit().await?;

    store.truncate_all_collections().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON, collections::CAMERA])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;
    let cameras = tx.readable_collection(collections::CAMERA)?;

    assert!(!people.contains_key("id-1").await?);
    assert!(!people.contains_key("id-2").await?);
    assert!(!cameras.contains_key("id-1").await?);

    Ok(())
}

#[async_test]
async fn test_get_all_values() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people.set("id-1", &Person::named("Amelia")).await?;
    people.set("id-2", &Person::named("Benjamin")).await?;
    people.set("id-3", &Person::named("Charlotte")).await?;
    people.set("id-4", &Person::named("Daniel")).await?;
    people.set("id-5", &Person::named("Emily")).await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;

    // Range
    {
        let values = people
            .get_all_values::<Person>(
                Query::from_range("id-2".."id-4"),
                QueryDirection::Forward,
                None,
            )
            .await?;

        assert_eq!(
            values,
            vec![Person::named("Benjamin"), Person::named("Charlotte"),]
        );
    }

    // RangeFrom
    {
        let values = people
            .get_all_values::<Person>(Query::from_range("id-3"..), QueryDirection::Forward, None)
            .await?;

        assert_eq!(
            values,
            vec![
                Person::named("Charlotte"),
                Person::named("Daniel"),
                Person::named("Emily"),
            ]
        );
    }

    // RangeFull
    {
        let values = people
            .get_all_values::<Person>(Query::<&str>::from_range(..), QueryDirection::Forward, None)
            .await?;

        assert_eq!(
            values,
            vec![
                Person::named("Amelia"),
                Person::named("Benjamin"),
                Person::named("Charlotte"),
                Person::named("Daniel"),
                Person::named("Emily"),
            ]
        );
    }

    // RangeInclusive
    {
        let values = people
            .get_all_values::<Person>(
                Query::from_range("id-2"..="id-4"),
                QueryDirection::Forward,
                None,
            )
            .await?;

        assert_eq!(
            values,
            vec![
                Person::named("Benjamin"),
                Person::named("Charlotte"),
                Person::named("Daniel"),
            ]
        );
    }

    // RangeTo
    {
        let values = people
            .get_all_values::<Person>(Query::from_range(.."id-3"), QueryDirection::Forward, None)
            .await?;

        assert_eq!(
            values,
            vec![Person::named("Amelia"), Person::named("Benjamin"),]
        );
    }

    // RangeToInclusive
    {
        let values = people
            .get_all_values::<Person>(Query::from_range(..="id-3"), QueryDirection::Forward, None)
            .await?;

        assert_eq!(
            values,
            vec![
                Person::named("Amelia"),
                Person::named("Benjamin"),
                Person::named("Charlotte"),
            ]
        );
    }

    // Only
    {
        let values = people
            .get_all_values::<Person>(Query::Only("id-3"), QueryDirection::Forward, None)
            .await?;

        assert_eq!(values, vec![Person::named("Charlotte"),]);
    }

    Ok(())
}

#[async_test]
async fn test_get_all_values_with_order_and_limit() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people.set("id-1", &Person::named("Amelia")).await?;
    people.set("id-2", &Person::named("Benjamin")).await?;
    people.set("id-3", &Person::named("Charlotte")).await?;
    people.set("id-4", &Person::named("Daniel")).await?;
    people.set("id-5", &Person::named("Emily")).await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;

    let values = people
        .get_all_values::<Person>(
            Query::from_range(..="id-4"),
            QueryDirection::Backward,
            Some(3),
        )
        .await?;

    assert_eq!(
        values,
        vec![
            Person::named("Daniel"),
            Person::named("Charlotte"),
            Person::named("Benjamin")
        ]
    );

    Ok(())
}

#[async_test]
async fn test_get_all_with_filter() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people.set("id-1", &Person::named("Amelia")).await?;
    people.set("id-2", &Person::named("Benjamin 1")).await?;
    people.set("id-3", &Person::named("Benjamin 2")).await?;
    people.set("id-4", &Person::named("Charlotte 1")).await?;
    people.set("id-5", &Person::named("Charlotte 2")).await?;
    people.set("id-6", &Person::named("Daniel 1")).await?;
    people.set("id-7", &Person::named("Daniel 2")).await?;
    people.set("id-8", &Person::named("Emily")).await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;

    let values = people
        .get_all_filtered::<Person, _>(
            Query::from_range(..="id-7"),
            QueryDirection::Backward,
            Some(3),
            |id, person| {
                if person.name.ends_with("1") {
                    return None;
                }
                Some((id, person.name))
            },
        )
        .await?;

    assert_eq!(
        values,
        vec![
            ("id-7".to_string(), "Daniel 2".to_string()),
            ("id-5".to_string(), "Charlotte 2".to_string()),
            ("id-3".to_string(), "Benjamin 2".to_string())
        ]
    );

    Ok(())
}

#[async_test]
async fn test_index_keys() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people
        .set(
            "id-1",
            &Person {
                name: "Amelia".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-2",
            &Person {
                name: "Benjamin".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-3",
            &Person {
                name: "Charlotte".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 03).unwrap(),
            },
        )
        .await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;
    let birthdays = people.index(collections::person::BIRTHDAY)?;

    let values = birthdays
        .get_all::<Person>(
            Query::from_range(NaiveDate::from_ymd_opt(2020, 01, 02).unwrap()..),
            QueryDirection::Backward,
            None,
        )
        .await?;

    assert_eq!(
        values,
        vec![
            (
                "id-3".to_string(),
                Person {
                    name: "Charlotte".to_string(),
                    birthday: NaiveDate::from_ymd_opt(2020, 01, 03).unwrap(),
                }
            ),
            (
                "id-2".to_string(),
                Person {
                    name: "Benjamin".to_string(),
                    birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
                }
            )
        ]
    );

    Ok(())
}

#[async_test]
async fn test_get_all_values_on_empty_collection() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;
    let people = tx.readable_collection(collections::PERSON)?;
    let values = people
        .get_all_values::<Person>(Query::<&str>::from_range(..), QueryDirection::Forward, None)
        .await?;

    assert_eq!(values, vec![]);

    Ok(())
}

#[async_test]
async fn test_index() -> Result<()> {
    let store = store().await?;

    let tx = store
        .transaction_for_reading_and_writing(&[collections::PERSON])
        .await?;
    let people = tx.writeable_collection(collections::PERSON)?;

    people
        .set(
            "id-1",
            &Person {
                name: "Amelia".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-2",
            &Person {
                name: "Benjamin".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-3",
            &Person {
                name: "Charlotte".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 03).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-4",
            &Person {
                name: "Daniel".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 04).unwrap(),
            },
        )
        .await?;
    people
        .set(
            "id-5",
            &Person {
                name: "Emily".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 05).unwrap(),
            },
        )
        .await?;

    tx.commit().await?;

    let tx = store
        .transaction_for_reading(&[collections::PERSON])
        .await?;

    let people = tx.readable_collection(collections::PERSON)?;
    let birthdays = people.index(collections::person::BIRTHDAY)?;

    let values = birthdays
        .get_all_values::<Person>(
            Query::from_range(
                NaiveDate::from_ymd_opt(2020, 01, 02).unwrap()
                    ..=NaiveDate::from_ymd_opt(2020, 01, 04).unwrap(),
            ),
            QueryDirection::Forward,
            None,
        )
        .await?;

    assert_eq!(
        values,
        vec![
            Person {
                name: "Benjamin".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 02).unwrap(),
            },
            Person {
                name: "Charlotte".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 03).unwrap(),
            },
            Person {
                name: "Daniel".to_string(),
                birthday: NaiveDate::from_ymd_opt(2020, 01, 04).unwrap(),
            }
        ]
    );

    Ok(())
}
