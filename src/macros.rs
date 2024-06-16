/// Creates the Database struct with the fitting logic. Use `open` for creating/opening one
/// at a specified `path` or use `open_in_memory` to only use one in memory.
///
/// Lock the `AtomicDatabase` using `read()` / `write()` to access and change it values.
/// The saving of changes will be applied after the used variables are dropped.
///
/// ```
/// use light_magic::db;
///
/// db! {
///     // `users` is the table name
///     // `{...}` is the table data
///     // the first field, like here `id`, is the `primary_key`
///     user => { id: usize, name: String, kind: String },
///     // using [...] after the table name you can add your own derives
///     // like here `PartialEq`
///     criminal: [PartialEq] => { user_name: String, entry: String }
/// }
/// ```
#[macro_export]
macro_rules! db {
    (
        $(
            $table:ident $(: [$($derive:ident),*])? => { $($field_name:ident : $field_ty:ty),* }
        ),* $(,)?
    ) => {
        use std::path::Path;
        use $crate::serde::{Serialize, Deserialize};
        use $crate::table::{Table, Matches, FirstField};
        use $crate::atomic::{DataStore, AtomicDatabase};
        use $crate::paste::paste;

        paste! {
            /// The Database Struct
            #[derive(Default, Debug, Serialize, Deserialize)]
            pub struct Database {
                $(
                    #[doc = "The " $table:camel " Table"]
                    [<$table:snake>]: Table<$crate::get_first_type!($($field_name : $field_ty),*), [<$table:camel>]>,
                )*
            }

            impl<'a> DataStore for Database {}

            $(
                #[derive(Default, Debug, Clone, Serialize, Deserialize $(, $($derive),* )?)]
                pub struct [<$table:camel>] {
                    $(
                        pub $field_name: $field_ty,
                    )*
                }

                impl Matches for [<$table:camel>] {
                    fn matches(&self, query: &str) -> bool {
                        $(
                            if format!("{:?}", self.$field_name).to_lowercase().contains(&query.to_lowercase()) {
                                return true;
                            }
                        )*
                        false
                    }
                }

                impl FirstField for [<$table:camel>] {
                    type FieldType = $crate::get_first_type!($($field_name : $field_ty),*);

                    fn first_field(&self) -> &Self::FieldType {
                        &$crate::get_first_name!(self, $($field_name),*)
                    }
                }
            )*
        }
    }
}

/// Helper for getting the first type of a struct
#[macro_export]
macro_rules! get_first_type {
    ($first_name:ident : $first_ty:ty, $($rest_name:ident : $rest_ty:ty),*) => {
        $first_ty
    };
    ($first_name:ident : $first_ty:ty) => {
        $first_ty
    };
}

/// Helper for getting the first name of a struct
#[macro_export]
macro_rules! get_first_name {
    ($value:expr, $first_name:ident, $($rest_name:ident),*) => {
        $value.$first_name
    };
    ($value:expr, $first_name:ident) => {
        $value.$first_name
    };
}

/// Joins Data in the Database together
///
/// ```
/// use light_magic::{db, join};
///
/// db! {
///     user => { id: usize, name: String, kind: String },
///     criminal => { user_name: String, entry: String }
/// }
///
/// let db = Database::open_in_memory();
/// // Firstly specify the Database which should be used, then the key,
/// // and lastly the joined items with the field which will be compared with the key
/// let joined = join!(db.read(), "Nils", user => name, criminal => user_name);
/// ```
#[macro_export]
macro_rules! join {
    ($db:expr, $key:expr, $($table:ident => $field:ident),* $(,)?) => {{
        $crate::paste::paste! {
            let mut combined_results = Vec::new();

            $(
                let [<$table _results>]: Vec<_> = $db.$table.values()
                    .filter(|val| val.$field == $key)
                    .cloned()
                    .collect();
            )*

            let len = vec![$([<$table _results>].len()),*].into_iter().min().unwrap_or(0);

            for i in 0..len {
                combined_results.push((
                    $([<$table _results>][i].clone(),)*
                ));
            }

            combined_results
        }
    }}
}
