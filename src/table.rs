use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use std::{clone::Clone, collections::btree_map::Values};

/// Represents a database table utilizing a `BTreeMap` for underlying data storage.
/// Offers enhanced methods for manipulating records, including `add`, `edit`, `delete`, `get`, and `search`.
#[serde_as]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Table<K, V>
where
    K: Ord + FromStr + Display,
    <K as FromStr>::Err: Display,
    V: Serialize + for<'a> Deserialize<'a>,
{
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    #[serde(flatten)]
    inner: BTreeMap<K, V>,
}

impl<K, V> Table<K, V>
where
    K: Clone + Ord + FromStr + Display,
    K::Err: Display,
    V: Clone + Matches + FirstField<FieldType = K> + Serialize + for<'a> Deserialize<'a>,
{
    /// Adds an entry to the table, returns the `value` or `None` if the addition failed
    pub fn add(&mut self, value: V) -> Option<V> {
        let key = value.first_field();
        if !self.inner.contains_key(key) {
            self.inner.insert(key.clone(), value.clone());
            return Some(value);
        }
        None
    }

    /// Gets an entry from the table, returns the `value` or `None` if it couldn't find the data
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Edits an entry in the table, returns the `new_value` or `None` if the editing failed
    pub fn edit(&mut self, key: &K, new_value: V) -> Option<V> {
        let new_key = new_value.first_field();
        if (key == new_key || !self.inner.contains_key(new_key)) && self.inner.remove(key).is_some()
        {
            self.inner.insert(new_key.clone(), new_value.clone());
            return Some(new_value);
        }
        None
    }

    /// Deletes an entry from the table, returns the `value` or `None` if the deletion failed
    pub fn delete(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    /// Searches the table by a `&str`, works in parallel
    pub fn search(&self, search: &str) -> Vec<&V> {
        self.inner
            .values()
            .filter(|val| val.matches(search))
            .collect()
    }

    /// Gets an iterator over the values of the map, in order by key.
    pub fn values(&self) -> Values<'_, K, V> {
        self.inner.values()
    }
}

/// Match trait, filled in the `db!` macro
pub trait Matches {
    fn matches(&self, search: &str) -> bool;
}

/// Trait for getting the value of the first field, filled in the `db!` macro
pub trait FirstField {
    type FieldType: Clone;
    fn first_field(&self) -> &Self::FieldType;
}

mod test {
    use serde::{Deserialize, Serialize};

    use super::{FirstField, Matches};

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    struct User {
        id: usize,
        name: String,
        age: usize,
    }

    impl Matches for User {
        fn matches(&self, _: &str) -> bool {
            false
        }
    }

    impl FirstField for User {
        type FieldType = usize;

        fn first_field(&self) -> &Self::FieldType {
            &self.id
        }
    }

    #[test]
    fn serialize() {
        use super::Table;

        let mut table = Table::default();

        table.add(User::default());

        serde_json::to_string(&table).unwrap();
        serde_json::from_str::<Table<usize, User>>(&"{\"0\":{\"id\":0,\"name\":\"\",\"age\":0}}")
            .unwrap();
    }
}
