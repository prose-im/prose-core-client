// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Macros --

#[macro_export]
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut map = std::collections::HashMap::new();

            $(
                map.insert($key, $value);
            )+

            map
        }
    };
);

// -- Exports --

pub use map;
