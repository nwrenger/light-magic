# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight, fast and easy-to-use implementation of a `persistent in-memory database`.

## Features

Please note that this database is highly optimized for read operations. Writing to the database is relatively slow when using `open` because each write operation involves writing data to the disk. These writes are done atomically, ensuring no data loss on a system-wide crash.

- **Persistent Data Storage**: Data can be saved automatically and persistently to a formatted `JSON` file via `open`, or it can be operated in-memory using `open_in_memory`.
- **Easy Table Markup**: Utilizes Rusts beautiful type system, structs and traits.
- **Powerful Data Access Functions**: Utilize functions like `search` and the `join!` macro for efficient data searching and joining.
- **Efficient Storage**: The database employs a custom `Table` data type, which uses the `BTreeMap` type from `std::collections` under the hood, for efficient storage and easy access of its tables.
- **Parallel Access Support**: Access the database in parallel using `Arc<AtomicDatabase<_>>`.

> ...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.6.0"
```

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
    user: Table<User>,
    permission: Table<Permission>,
    criminal: Table<Criminal>,
    settings: Settings,
}

impl DataStore for Database {}

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

     db.write().user.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    println!("{:?}", db.read().user.get(&0));
    println!("{:?}", db.read().user.search(|user| { user.name.contains("Nils") }));

    db.write().permission.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    println!("{:?}", db.read().permission.get(&String::from("Nils")));
    println!("{:?}", db.read().permission.search(|permission| { permission.level == Level::Admin }));

    db.write().criminal.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    println!("{:?}", db.read().criminal.get(&String::from("Nils")));
    println!("{:?}", db.read().criminal.search(|criminal| { criminal.entry.contains("No records") }));

    db.write().settings = Settings {
        time: 1718744090,
        password: String::from("password"),
    };
    println!("{:?}", db.read().settings);

    let joined = join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    println!("{:?}", joined);
}
```

## Todos

> None currently
