/// Creates the Database struct with the fitting logic
///
/// ## Concurrency
///
/// Lock the AtomicDatabase using read() / write() to access and changes it values.
/// The saving of changes will be applied after the used variables are dropped.
///
/// ## Functions
///
/// - `add_#table`
/// - `get_#table`
/// - `edit_#table`
/// - `delete_#table`
/// - `search_#table`
///
/// ## Example
/// ```
/// use light_magic::db;
///
/// db! {
///     // `users` is the table name
///     // `{...}` is the table data
///     // the first field, like here `name`, is the `primary_key`
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
        use std::collections::BTreeMap;
        use serde::{Serialize, Deserialize};
        use std::path::Path;

        $crate::paste::paste! {
            /// The Database Struct
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct Database {
                $(
                    #[doc = "The " $table:camel " Table, never access this directly and use the functions on the `Database`"]
                    [<$table:snake>]: BTreeMap<$crate::get_first_type!($($field_name : $field_ty),*), [<$table:camel>]>,
                )*
            }

            impl<'a> $crate::persistence::DB<'a> for Database {}

            impl Database {
                #[doc = "Make a new Database Instance"]
                pub fn new(db: &Path) -> $crate::persistence::AtomicDatabase<Database> {
                    if db.exists() {
                        $crate::persistence::AtomicDatabase::load(&db).unwrap()
                    } else {
                        $crate::persistence::AtomicDatabase::create(&db).unwrap()
                    }
                }

                $(
                    #[doc = "Adds a " $table:snake ", returns the `value` or `None` if the addition failed"]
                    pub fn [<add_ $table:snake>](&mut self, value: [<$table:camel>]) -> Option<[<$table:camel>]> {
                        if self.[<$table:snake>].get(&$crate::get_first_name!(value, $($field_name),*)).is_none() {
                            self.[<$table:snake>].insert((&$crate::get_first_name!(value, $($field_name),*)).clone(), value.clone());
                            return Some(value);
                        }
                        None
                    }

                    #[doc = "Gets a " $table:snake ", returns the `value` or `None` if it couldn't find the data"]
                    pub fn [<get_ $table:snake>](&self, key: &$crate::get_first_type!($($field_name : $field_ty),*)) -> Option<[<$table:camel>]> {
                        self.[<$table:snake>].get(key).cloned()
                    }

                    #[doc = "Edits a " $table:snake ", returns the `new_value` or `None` if the editing failed"]
                    pub fn [<edit_ $table:snake>](&mut self, key: &$crate::get_first_type!($($field_name : $field_ty),*), new_value: [<$table:camel>]) -> Option<[<$table:camel>]> {
                        if &$crate::get_first_name!(new_value, $($field_name),*) == key || self.[<$table:snake>].get(&$crate::get_first_name!(new_value, $($field_name),*)).is_none() {
                            if self.[<$table:snake>].remove(key).is_some() {
                                self.[<$table:snake>].insert(($crate::get_first_name!(new_value, $($field_name),*)).clone(), new_value.clone());
                                return Some(new_value);
                            }
                        }
                        None
                    }

                    #[doc = "Deletes a " $table:snake ", returns the `value` or `None` if the deletion failed"]
                    pub fn [<delete_ $table:snake>](&mut self, key: &$crate::get_first_type!($($field_name : $field_ty),*)) -> Option<[<$table:camel>]> {
                        self.[<$table:snake>].remove(key)
                    }

                    #[doc = "Searches the " $table:camel " Table by a `&str`, works in parallel"]
                    pub fn [<search_ $table:snake>](&self, search: &str) -> Vec<[<$table:camel>]> {
                        self.[<$table:snake>].iter().map(|(_, val)| val.clone()).filter(|val| val.matches(search)).collect()
                    }
                )*
            }

            $(
                #[derive(Default, Debug, Clone, Serialize, Deserialize $(, $($derive),* )?)]
                pub struct [<$table:camel>] {
                    $(
                        pub $field_name: $field_ty,
                    )*
                }

                impl [<$table:camel>] {
                    pub fn matches(&self, query: &str) -> bool {
                        $(
                            let val = format!("{:?}", self.$field_name).to_lowercase();
                            if val.contains(&query.to_lowercase()) {
                                return true;
                            }
                        )*
                        false
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
/// ## Example
/// ```
/// use light_magic::{db, join};
///
/// db! {
///     user => { id: usize, name: String, kind: String },
///     permission => { user_name: String, level: Level },
///     criminal => { user_name: String, entry: String }
/// }
///
/// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
/// enum Level {
///     #[default]
///     Admin,
/// }
///
/// let db = Database::new(Path::new("./tests/test1.json"));
/// // Firstly specify the db to be used, then the key,
/// // and lastly the joined items with the field which should be joined
/// let joined = join!(db.read(), "Nils", user => name, criminal => user_name);
/// ```
#[macro_export]
macro_rules! join {
    ($db:expr, $key:expr, $($table:ident => $field:ident),* $(,)?) => {{
        (
            $(
                {
                    let filtered = $db.$table.iter().map(|(_, val)| val.clone()).filter(|val| val.$field == $key).collect::<Vec<_>>();
                    if !filtered.is_empty() {
                        Some(filtered)
                    } else {
                        None
                    }
                }
            ),*
        )
    }};
}
