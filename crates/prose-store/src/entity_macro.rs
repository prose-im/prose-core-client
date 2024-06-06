#[macro_export]
macro_rules! define_entity {
    // Case where ID type and indexes are specified
    ($struct_name:ident, $collection_name:expr, $id_type:ty, $($idx_name:ident => { columns: [$($col:expr),+], unique: $unique:expr }),*) => {
        impl $struct_name {
            $(
                pub fn $idx_name() -> [&'static str; { let mut count = 0; $(let _ = $col; count += 1;)+ count }] {
                    [$($col),+]
                }
            )*
        }

        impl Entity for $struct_name {
            type ID = $id_type;

            fn id(&self) -> &Self::ID {
                &self.id
            }

            fn collection() -> &'static str {
                $collection_name
            }

            fn indexes() -> Vec<IndexSpec> {
                let mut indexes = Vec::new();
                $(
                    {
                        let mut builder = IndexSpec::builder();
                        $(
                            builder = builder.add_column($col);
                        )+
                        if $unique {
                            builder = builder.unique();
                        }
                        indexes.push(builder.build());
                    }
                )*
                indexes
            }
        }
    };

    // Case where indexes are not specified, default to no indexes
    ($struct_name:ident, $collection_name:expr, $id_type:ty) => {
        define_entity!($struct_name, $collection_name, $id_type, );
    };

    // Case where ID type is not specified, default to String
    ($struct_name:ident, $collection_name:expr, $($idx_name:ident => { columns: [$($col:expr),+], unique: $unique:expr }),*) => {
        define_entity!($struct_name, $collection_name, String, $($idx_name => { columns: [$($col),+], unique: $unique }),*);
    };

    // Case where neither ID type nor indexes are specified
    ($struct_name:ident, $collection_name:expr) => {
        define_entity!($struct_name, $collection_name, String, );
    };
}
