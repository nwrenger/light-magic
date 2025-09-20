# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight, fast and easy-to-use implementation of a `persistent or optionally encrypted in-memory database`.

## Features

Please note that this database is highly optimized for read operations. Writing to the database is relatively slow when using `open` because each write operation involves writing data to the disk. These writes are done atomically, ensuring no data loss on a system-wide crash.

- **Persistent Data Storage**: Data can be saved automatically and persistently to a formatted `JSON` file via `open`, or it can be operated in-memory using `open_in_memory`.
- **Encrypted Persistent Data Storage**: Data can be also saved encrypted via the `encrypted` module using the same `open` method.
- **Easy Table Markup**: Utilizes Rusts beautiful type system, structs and traits.
- **Powerful Data Access Functions**: Utilize functions like `search` / `search_ordered` and the `join!` macro for efficient data searching and joining.
- **Efficient Storage**: The database employs a custom `Table` data type, which uses the `BTreeMap` type from `std::collections` under the hood, for efficient storage and easy access of its tables.
- **Parallel Access Support**: Access the database in parallel using `Arc<AtomicDatabase<_>>`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.7.2"
```

## Feature Flags

`light-magic` is feature-flag driven. By default, only the `atomic` module is enabled.
You can enable additional functionality in your `Cargo.toml`:

- `atomic`: _Enabled by default_. Provides the basic atomic database with persistent JSON storage, type-safe tables, and the `DataStore` trait.
- `encrypted`: Enables the `encrypted` module, adding Argon2id password-based key derivation, AES-256-GCM authenticated encryption (96-bit nonces), and compact bincode serialization on top of the atomic database.

## Examples

Using it in an `axum` Server? Look here: [maud-magic-rs](https://github.com/nwrenger/maud-magic-rs). Otherwise, look at this general example:

```rust
use light_magic::{
    atomic::DataStore,
    join,
    serde::{Deserialize, Serialize},
    table::{PrimaryKey, Table},
};

#[derive(Default, Debug, Serialize, Deserialize)]
struct Database {
    users: Table<User>,
    permissions: Table<Permission>,
    criminals: Table<Criminal>,
    settings: Settings,
}

impl light_magic::atomic::DataStore for Database {}
// or with features = ["encrypted"]
// impl light_magic::encrypted::EncryptedDataStore for Database {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: usize,
    name: String,
    kind: String,
}

impl PrimaryKey for User {
    type PrimaryKeyType = usize;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.id
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Permission {
    user_name: String,
    level: Level,
}

impl PrimaryKey for Permission {
    type PrimaryKeyType = String;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.user_name
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Level {
    #[default]
    Admin,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Criminal {
    user_name: String,
    entry: String,
}

impl PrimaryKey for Criminal {
    type PrimaryKeyType = String;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.user_name
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Settings {
    time: usize,
    password: String,
}

fn main() {
    let db = Database::open("./tests/test.json");
    // or with features = ["encrypted"]
    // let db = Database::open("./tests/test.json", "somePassword");

     db.write().users.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    println!("{:?}", db.read().users.get(&0));
    println!("{:?}", db.read().users.search(|user| { user.name.contains("Nils") }));

    db.write().permissions.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    println!("{:?}", db.read().permissions.get(&String::from("Nils")));
    println!("{:?}", db.read().permissions.search(|permission| { permission.level == Level::Admin }));

    db.write().criminals.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    println!("{:?}", db.read().criminals.get(&String::from("Nils")));
    println!("{:?}", db.read().criminals.search(|criminal| { criminal.entry.contains("No records") }));

    db.write().settings = Settings {
        time: 1718744090,
        password: String::from("password"),
    };
    println!("{:?}", db.read().settings);

    let joined = join!(db.read(), "Nils", users => name, permissions => user_name, criminals => user_name);
    println!("{:?}", joined);
}
```
