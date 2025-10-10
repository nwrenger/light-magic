use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::str::FromStr;
use std::{
    clone::Clone,
    collections::btree_map::{Values, ValuesMut},
};

/// Trait for getting the value of the primary key
pub trait PrimaryKey {
    type PrimaryKeyType;
    fn primary_key(&self) -> &Self::PrimaryKeyType;
}

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
#[derive(Default, Debug, Clone)]
pub struct Table<V>
where
    V: PrimaryKey + Serialize,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    inner: BTreeMap<<V as PrimaryKey>::PrimaryKeyType, V>,
}

impl<V> Serialize for Table<V>
where
    V: PrimaryKey + Serialize + for<'a> Deserialize<'a>,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            // Human-readable: emit as a map<String, V>
            let mut map = serializer.serialize_map(Some(self.inner.len()))?;
            for (k, v) in &self.inner {
                map.serialize_entry(&k.to_string(), v)?;
            }
            map.end()
        } else {
            // Binary (e.g., bincode): emit as a sequence of V with known length
            let mut seq = serializer.serialize_seq(Some(self.inner.len()))?;
            for v in self.inner.values() {
                seq.serialize_element(v)?;
            }
            seq.end()
        }
    }
}

impl<'de, V> Deserialize<'de> for Table<V>
where
    V: PrimaryKey + Serialize + Deserialize<'de>,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            // Human-readable: expect a map<String, V>
            struct MapVisitor<V>(PhantomData<V>);

            impl<'de, V> Visitor<'de> for MapVisitor<V>
            where
                V: PrimaryKey + Serialize + Deserialize<'de>,
                V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
                <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
            {
                type Value = Table<V>;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("a map of stringified primary keys to rows")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut inner = BTreeMap::new();
                    while let Some((k_str, v)) = map.next_entry::<String, V>()? {
                        let k = V::PrimaryKeyType::from_str(&k_str).map_err(|e| {
                            A::Error::custom(format!(
                                "failed to parse primary key '{}': {}",
                                k_str, e
                            ))
                        })?;
                        // Optional: sanity check that v.primary_key() matches k
                        inner.insert(k, v);
                    }
                    Ok(Table { inner })
                }
            }

            deserializer.deserialize_map(MapVisitor::<V>(PhantomData))
        } else {
            // Binary: expect a sequence of V; rebuild keys from PrimaryKey
            struct SeqVisitor<V>(PhantomData<V>);

            impl<'de, V> Visitor<'de> for SeqVisitor<V>
            where
                V: PrimaryKey + Serialize + Deserialize<'de>,
                V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
                <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
            {
                type Value = Table<V>;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("a sequence of table rows")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut inner = BTreeMap::new();
                    while let Some(v) = seq.next_element::<V>()? {
                        let k = v.primary_key().clone();
                        inner.insert(k, v);
                    }
                    Ok(Table { inner })
                }
            }

            deserializer.deserialize_seq(SeqVisitor::<V>(PhantomData))
        }
    }
}

impl<V> Table<V>
where
    V: PrimaryKey + Serialize + for<'a> Deserialize<'a>,
    V::PrimaryKeyType: Ord + FromStr + Display + Debug + Clone,
    <<V as PrimaryKey>::PrimaryKeyType as FromStr>::Err: std::fmt::Display,
{
    /// Adds an entry to the table, returns the `value` or `None` if the `key` already exists in that table.
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

    /// Gets an entry from the table, returns the `value` or `None` if it couldn't find the `value`.
    pub fn get(&self, key: &V::PrimaryKeyType) -> Option<&V> {
        self.inner.get(key)
    }

    /// Gets a mutable entry from the table, returns the `value` or `None` if it couldn't find the `value`.
    pub fn get_mut(&mut self, key: &V::PrimaryKeyType) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Edits an entry in the table, returns the `new_value` or `None` if the entry couldn't be found.
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

    /// Deletes an entry from the table, returns the `value` or `None` if the `key` wasn't found.
    pub fn delete(&mut self, key: &V::PrimaryKeyType) -> Option<V> {
        self.inner.remove(key)
    }

    /// Searches the table by a predicate function.
    pub fn search<F>(&self, predicate: F) -> Vec<&V>
    where
        F: Fn(&V) -> bool,
    {
        self.inner.values().filter(|&val| predicate(val)).collect()
    }

    /// Searches the table by a predicate function and a custom ordering with a comparator function.
    pub fn search_ordered<F, O>(&self, predicate: F, comparator: O) -> Vec<&V>
    where
        F: Fn(&V) -> bool,
        O: Fn(&&V, &&V) -> std::cmp::Ordering,
    {
        let mut result = self.search(predicate);
        result.sort_by(comparator);
        result
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

#[cfg(test)]
mod test {
    use super::{PrimaryKey, Table};
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    fn json_roundtrip_as_map() {
        let mut table = Table::default();
        table.add(User {
            id: 0,
            name: "".into(),
            age: 0,
        });
        let s = serde_json::to_string(&table).unwrap();
        assert_eq!(s, r#"{"0":{"id":0,"name":"","age":0}}"#);
        let back: Table<User> = serde_json::from_str(&s).unwrap();
        assert!(back.get(&0).is_some());
    }

    #[test]
    #[cfg(feature = "encrypted")]
    fn bincode_roundtrip_as_seq() {
        use crate::encrypted::bincode_cfg;

        let mut table = Table::default();
        for i in 0..3 {
            table.add(User {
                id: i,
                name: format!("u{i}"),
                age: i,
            });
        }
        let bytes = bincode::serde::encode_to_vec(&table, bincode_cfg()).unwrap();
        let (back, _): (Table<User>, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode_cfg()).unwrap();
        assert_eq!(table.values().count(), back.values().count());
        for i in 0..3 {
            assert_eq!(table.get(&i).unwrap().name, back.get(&i).unwrap().name);
        }
    }
}
