// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// use super::models::accounts::Account;

// use mdl::{Cache, Model};

// TODO: example
// fn end_to_end_cache() {
//     // initializing the cache. This str will be the fs persistence path
//     let db = "/tmp/mydb.lmdb";
//     let cache = Cache::new(db).unwrap();
//
//     // create a new *object* and storing in the cache
//     let a = Account {
//         p1: "hello".to_string(),
//         p2: 42,
//     };
//     let r = a.store(&cache);
//     assert!(r.is_ok());
//
//     // querying the cache by key and getting a new *instance*
//     let a1: Account = Account::get(&cache, "hello:42").unwrap();
//     assert_eq!(a1.p1, a.p1);
//     assert_eq!(a1.p2, a.p2);
// }
