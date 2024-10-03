use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::{
    clone::Clone,
    collections::btree_map::{Values, ValuesMut},
};

/// Represents a database table utilizing a `BTreeMap` for underlying data storage.
/// Needs the `PrimaryKey` trait to be implemented for the value type. Offers
/// enhanced methods for manipulating records, including `add`, `edit`, `delete`, `get`, and `search`.
/// ```
/// use light_magic::{
///     serde::{Deserialize, Serialize},
///     table::{PrimaryKey, Table},
/// };
///
/// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
/// struct User {
///     id: usize,
///     name: String,
///     age: usize,
/// }
///
/// impl PrimaryKey for User {
///     type PrimaryKeyType = usize;
///
///     fn primary_key(&self) -> &Self::PrimaryKeyType {
///         &self.id
///     }
/// }
/// ```
#[serde_as]
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Table<V>
where
    V: PrimaryKey + Serialize + for<'a> Deserialize<'a>,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    #[serde(flatten)]
    inner: BTreeMap<<V as PrimaryKey>::PrimaryKeyType, V>,
}

impl<V> Table<V>
where
    V: PrimaryKey + Serialize + for<'a> Deserialize<'a>,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    /// Adds an entry to the table, returns the `value` or `None` if the addition failed
    pub fn add(&mut self, value: V) -> Option<V>
    where
        V: Clone,
        V::PrimaryKeyType: Clone,
    {
        let key = value.primary_key();
        if !self.inner.contains_key(key) {
            self.inner.insert(key.clone(), value.clone());
            return Some(value);
        }
        None
    }

    /// Gets an entry from the table, returns the `value` or `None` if it couldn't find the data
    pub fn get(&self, key: &V::PrimaryKeyType) -> Option<&V> {
        self.inner.get(key)
    }

    /// Gets an mutable entry from the table, returns the `value` or `None` if it couldn't find the data
    pub fn get_mut(&mut self, key: &V::PrimaryKeyType) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Edits an entry in the table, returns the `new_value` or `None` if the editing failed
    pub fn edit(&mut self, key: &V::PrimaryKeyType, new_value: V) -> Option<V>
    where
        V: Clone,
        V::PrimaryKeyType: Clone,
    {
        let new_key = new_value.primary_key();
        if (key == new_key || !self.inner.contains_key(new_key)) && self.inner.remove(key).is_some()
        {
            self.inner.insert(new_key.clone(), new_value.clone());
            return Some(new_value);
        }
        None
    }

    /// Deletes an entry from the table, returns the `value` or `None` if the deletion failed
    pub fn delete(&mut self, key: &V::PrimaryKeyType) -> Option<V> {
        self.inner.remove(key)
    }

    /// Searches the table by a predicate function
    pub fn search<F>(&self, predicate: F) -> Vec<&V>
    where
        F: Fn(&V) -> bool,
    {
        self.inner.values().filter(|&val| predicate(val)).collect()
    }

    /// Gets an iterator over the values of the map, in order by key.
    pub fn values(&self) -> Values<'_, V::PrimaryKeyType, V> {
        self.inner.values()
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    pub fn values_mut(&mut self) -> ValuesMut<'_, V::PrimaryKeyType, V> {
        self.inner.values_mut()
    }
}

/// Trait for getting the value of the primary key
pub trait PrimaryKey {
    type PrimaryKeyType;
    fn primary_key(&self) -> &Self::PrimaryKeyType;
}

mod test {
    use serde::{Deserialize, Serialize};

    use super::PrimaryKey;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    struct User {
        id: usize,
        name: String,
        age: usize,
    }

    impl PrimaryKey for User {
        type PrimaryKeyType = usize;

        fn primary_key(&self) -> &Self::PrimaryKeyType {
            &self.id
        }
    }

    #[test]
    fn serialize() {
        use super::Table;

        let mut table = Table::default();

        table.add(User::default());

        serde_json::to_string(&table).unwrap();
        serde_json::from_str::<Table<User>>(&"{\"0\":{\"id\":0,\"name\":\"\",\"age\":0}}").unwrap();
    }
}
