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
///     // `Table` is the identifier that this will use the builtin table type
///     // `User` is the table name
///     // `{...}` is the table data
///     // the first field, like here `id`, is the `primary_key`
///     Table<User> => { id: usize, name: String, kind: String },
///     // to not use the builtin table type use `None` as the identifier of the table
///     // using `:` after the table name you can add your own derives
///     // like here `PartialEq`
///     None<Criminal: PartialEq> => { user_name: String, entry: String }
/// }
/// ```
#[macro_export]
macro_rules! db {
    (
        $(
            $table_ty:ident<$table:ty $( : $($derive:ident),* )?> => {
                $($field_name:ident : $field_ty:ty),*
            }
        ),* $(,)?
    ) => {
        use std::path::Path;
        use $crate::serde::{Serialize, Deserialize};
        use $crate::atomic::{DataStore, AtomicDatabase};
        use $crate::paste::paste;

        paste! {
            /// The Database Struct
            #[derive(Default, Debug, Serialize, Deserialize)]
            pub struct Database {
                $(
                        #[doc = "The " $table:camel " Table"]
                        pub [<$table:snake>]: db!(@expand_table_ty $table_ty, db!(@get_first_type $($field_name : $field_ty),*), [<$table:camel>]),
                )*
            }

            impl<'a> DataStore for Database {}

            $(
                #[derive(Default, Debug, Clone, Serialize, Deserialize, $( $( $derive),*)?)]
                pub struct [<$table:camel>] {
                    $(
                        pub $field_name: $field_ty,
                    )*
                }

                db!(@impls $table_ty, [<$table:camel>], $($field_name : $field_ty),*);
            )*
        }
    };

    // Only creating impls if using a specific table type
    (@impls Table, $table_name:ident, $($field_name:ident : $field_ty:ty),*) => {
        impl $crate::table::Matches for $table_name {
            fn matches(&self, query: &str) -> bool {
                $(
                    if format!("{:?}", self.$field_name).to_lowercase().contains(&query.to_lowercase()) {
                        return true;
                    }
                )*
                false
            }
        }

        impl $crate::table::FirstField for $table_name {
            type FieldType = db!(@get_first_type $($field_name : $field_ty),*);

            fn first_field(&self) -> &Self::FieldType {
                &db!(@get_first_name self, $($field_name),*)
            }
        }
    };

    // If you don't want to use the build in type
    (@impls None, $table_name:ident, $($field_name:ident : $field_ty:ty),*) => {};

    // Helper for expanding the table type conditionally
    (@expand_table_ty Table, $first_type:ty, $table_name:ident) => {
        $crate::table::Table<$first_type, $table_name>
    };
    (@expand_table_ty None, $first_type:ty, $table_name:ident) => {
        $table_name
    };

    // Helper for getting the first name of a struct
    (@get_first_name $value:expr, $first_name:ident, $($rest_name:ident),*) => {
        $value.$first_name
    };
    (@get_first_name $value:expr, $first_name:ident) => {
        $value.$first_name
    };

    // Helper for getting the first type of a struct
    (@get_first_type $first_name:ident : $first_ty:ty, $($rest_name:ident : $rest_ty:ty),*) => {
        $first_ty
    };
    (@get_first_type $first_name:ident : $first_ty:ty) => {
        $first_ty
    };
}

/// Joins Data of different `Tables` in the Database together
///
/// ```
/// use light_magic::{db, join};
///
/// db! {
///     Table<User> => { id: usize, name: String, kind: String },
///     Table<Criminal> => { user_name: String, entry: String }
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
