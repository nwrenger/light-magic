# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight, fast and easy-to-use implementation of a `persistent in-memory database`.

## Features

- Please note that this database is highly optimized for read operations. Writing to the database is relatively slow because each write operation involves writing data to the disk.
- Writes to the disk are done atomically meaning no data loss on a system-wide crash.
- This crate utilizes the `BTreeMap` from `std::collections` for storing and accessing it's tables.
- Easy markup of tables using the `db!` macro.
- Useful data accessing functions like `search` or `join!` macro to search the data or join data together.
- Can save data to a specific path after each change automatically, atomically, and persistently via `open` or not via `open_in_memory`.
- Supports accessing the database in parallel using a `Arc<AtomicDatabase<_>>`.

...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.5.1"
```

## Examples

Using it in an `axum` Server? Look here: [maud-magic-rs](https://github.com/nwrenger/maud-magic-rs). Otherwise, look at this general example:

```rust
use light_magic::{db, join};

db! {
    user => { id: usize, name: String, kind: String },
    permission => { user_name: String, level: Level },
    criminal => { user_name: String, entry: String  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
enum Level {
    #[default]
    Admin,
}

fn main() {
    let db = Database::open("./tests/test.json");

     db.write().user.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    println!("{:?}", db.read().user.get(&0));
    println!("{:?}", db.read().user.search("0"));

    db.write().permission.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    println!("{:?}", db.read().permission.get(&String::from("Nils")));
    println!("{:?}", db.read().permission.search("Admin"));

    db.write().criminal.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    println!("{:?}", db.read().criminal.get(&String::from("Nils")));
    println!("{:?}", db.read().criminal.search("No records"));

    let joined = join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    println!("{:?}", joined);
}
```

## Todos

> None currently
