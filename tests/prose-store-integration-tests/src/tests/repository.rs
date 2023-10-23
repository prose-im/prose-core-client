use crate::tests::{async_test, platform_driver, PlatformDriver};
use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_store::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[entity]
struct User {
    id: BareJid,
    name: String,
    company: Option<Company>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Company {
    name: String,
    role: String,
}

impl User {
    fn developer() -> Self {
        Self {
            id: BareJid::from_str("developer@prose.org").unwrap(),
            name: "Señor Developer".to_string(),
            company: Company {
                name: "Prose Foundation".to_string(),
                role: "Developer".to_string(),
            }
            .into(),
        }
    }

    fn designer() -> Self {
        Self {
            id: BareJid::from_str("designer@prose.org").unwrap(),
            name: "Picasso Pixel".to_string(),
            company: Some(Company {
                name: "Prose Foundation".to_string(),
                role: "Designer".to_string(),
            }),
        }
    }

    fn tester() -> Self {
        Self {
            id: BareJid::from_str("tester@prose.org").unwrap(),
            name: "Bugslayer".to_string(),
            company: Some(Company {
                name: "Prose Foundation".to_string(),
                role: "Tester".to_string(),
            }),
        }
    }

    fn applicant() -> Self {
        Self {
            id: BareJid::from_str("applicant@prose.org").unwrap(),
            name: "Eager Beaver".to_string(),
            company: None,
        }
    }

    fn hired_applicant() -> Self {
        let mut applicant = Self::applicant();
        applicant.company = Some(Company::junior_developer());
        applicant
    }
}

impl Company {
    fn junior_developer() -> Self {
        Company {
            name: "Prose Foundation".to_string(),
            role: "Junior Developer".to_string(),
        }
    }
}

async fn store() -> Result<Store<PlatformDriver>> {
    let store = Store::open(
        platform_driver(
            std::path::Path::new(file!())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        ),
        1,
        |event| {
            event.tx.create_collection(User::collection())?;
            Ok(())
        },
    )
    .await?;
    store.truncate_all_collections().await?;
    Ok(store)
}

#[async_test]
async fn test_get() -> Result<()> {
    let repo = Repository::<_, User>::new(store().await?);

    repo.put(&User::developer()).await?;
    repo.put(&User::designer()).await?;
    repo.put(&User::tester()).await?;

    assert_eq!(
        repo.get(&User::developer().id).await?,
        Some(User::developer())
    );
    assert_eq!(
        repo.get(&User::designer().id).await?,
        Some(User::designer())
    );
    assert_eq!(repo.get(&User::tester().id).await?, Some(User::tester()));

    Ok(())
}

#[async_test]
async fn test_get_all() -> Result<()> {
    let repo = Repository::<_, User>::new(store().await?);

    repo.put(&User::developer()).await?;
    repo.put(&User::designer()).await?;
    repo.put(&User::tester()).await?;

    assert_eq!(
        repo.get_all().await?,
        vec![User::designer(), User::developer(), User::tester()]
    );

    Ok(())
}

#[async_test]
async fn test_delete() -> Result<()> {
    let repo = Repository::<_, User>::new(store().await?);

    repo.put(&User::developer()).await?;
    repo.put(&User::designer()).await?;
    repo.put(&User::tester()).await?;

    assert_eq!(
        repo.get_all().await?,
        vec![User::designer(), User::developer(), User::tester()]
    );

    repo.delete(&User::designer().id).await?;

    assert_eq!(repo.get(&User::designer().id).await?, None);
    assert_eq!(
        repo.get_all().await?,
        vec![User::developer(), User::tester()]
    );

    Ok(())
}

#[async_test]
async fn test_update_entry() -> Result<()> {
    let repo = Repository::<_, User>::new(store().await?);

    assert_eq!(repo.get(&User::applicant().id).await?, None);

    repo.entry(&User::applicant().id)
        .insert_if_needed(User::applicant())
        .and_update(|user| user.company = Some(Company::junior_developer()))
        .await?;

    assert_eq!(
        repo.get(&User::applicant().id).await?,
        Some(User::hired_applicant())
    );

    repo.entry(&User::applicant().id)
        .insert_if_needed_with(|_| panic!("Should not be called"))
        .and_update(|user| user.company = None)
        .await?;

    assert_eq!(
        repo.get(&User::applicant().id).await?,
        Some(User::applicant())
    );

    Ok(())
}

#[async_test]
async fn test_update_entry_in_async_trait_compiles() -> Result<()> {
    let wrapper = RepoWrapper1::new().await?;
    wrapper.update_company(Company::junior_developer()).await?;

    let wrapper = RepoWrapper2::new().await?;
    wrapper.update_company(Company::junior_developer()).await?;

    Ok(())
}

#[async_trait]
trait RepoWrapperTrait {
    async fn update_company(&self, company: Company) -> Result<()>;
}

struct RepoWrapper1 {
    repo: Repository<PlatformDriver, User>,
}

impl RepoWrapper1 {
    async fn new() -> Result<Self> {
        Ok(Self {
            repo: Repository::new(store().await?),
        })
    }
}

struct RepoWrapper2 {
    store: Store<PlatformDriver>,
}

impl RepoWrapper2 {
    async fn new() -> Result<Self> {
        Ok(Self {
            store: store().await?,
        })
    }
}

// This code doesn't compile due to a 'higher-ranked lifetime error'.
// See: https://github.com/rust-lang/rust/issues/102211

// #[async_trait]
// impl RepoWrapperTrait for RepoWrapper1 {
//     async fn update_company(&self, company: Company) -> Result<()> {
//         self.repo
//             .entry(&User::applicant().id)
//             .insert_if_needed(User::applicant())
//             .and_update(|user| user.company = Some(company))
//             .await?;
//         Ok(())
//     }
// }

#[async_trait]
impl RepoWrapperTrait for RepoWrapper1 {
    async fn update_company(&self, company: Company) -> Result<()> {
        // This is the ugly workaround for the code above which doesn't compile.
        let tx = self
            .repo
            .store()
            .transaction_for_reading_and_writing(&[User::collection()])
            .await?;
        let collection = tx.writeable_collection(User::collection())?;
        let id = User::applicant().id;
        let mut user = collection
            .get::<_, User>(&id)
            .await?
            .unwrap_or_else(|| User::applicant());
        user.company = Some(company);
        collection.put(&id, &user)?;
        Ok(())
    }
}

#[async_trait]
impl RepoWrapperTrait for RepoWrapper2 {
    // This is a slightly more elegant workaround for the code above which doesn't compile.
    async fn update_company(&self, company: Company) -> Result<()> {
        upsert!(
            User,
            store: self.store,
            id: &User::applicant().id,
            insert_if_needed: || User::applicant(),
            update: |user: &mut User| user.company = Some(company)
        );
        Ok(())
    }
}
