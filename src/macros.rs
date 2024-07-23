/// Joins Data of different `Tables` in the Database together
///
/// ```
/// use light_magic::{
///     atomic::DataStore,
///     join,
///     serde::{Deserialize, Serialize},
///     table::{PrimaryKey, Table},
/// };
///
/// #[derive(Default, Debug, Serialize, Deserialize)]
/// struct Database {
///    user: Table<User>,
///    criminal: Table<Criminal>,
/// }
///
/// impl DataStore for Database {}
///
/// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
/// struct User {
///     id: usize,
///     name: String,
///     kind: String,
/// }
///
/// impl PrimaryKey for User {
///     type PrimaryKeyType = usize;
///
///     fn primary_key(&self) -> &Self::PrimaryKeyType {
///         &self.id
///     }
/// }
///
/// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
/// struct Criminal {
///     user_name: String,
///     entry: String,
/// }
///
/// impl PrimaryKey for Criminal {
///     type PrimaryKeyType = String;
///
///     fn primary_key(&self) -> &Self::PrimaryKeyType {
///         &self.user_name
///     }
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
