# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight and easy-to-use implementation of a persistent `in-memory database`.

## Features

- This crate utilizes the `BTreeMap` from `std::collections` for storing and accessing it's data.
- Easy markup of tables using the `db!` macro
- Useful data accessing functions like `search` or `join!` macro to search the data or join data together
- Can save data to a specific path after each change automatically, atomically, and persistently via `AtomicDatabase<_>`
- Supports accessing the database in parallel using a `Arc<AtomicDatabase<_>>`

...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.4.0"
```

## Examples

Using it in an Axum Server? Look here: [maud-magic-rs](https://github.com/nwrenger/maud-magic-rs). Otherwise, look at this general example:

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
    let db = Database::new(Path::new("./tests/test1.json"));

     db.write().add_user(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    println!("{:?}", db.read().get_user(&0));
    println!("{:?}", db.read().search_user("0"));

    db.write().add_permission(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    println!("{:?}", db.read().get_permission(&String::from("Nils")));
    println!("{:?}", db.read().search_permission("Admin"));

    db.write().add_criminal(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    println!("{:?}", db.read().get_criminal(&String::from("Nils")));
    println!("{:?}", db.read().search_criminal("No records"));

    let joined = join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    println!("{:?}", joined);
}
```

## Todos

> None currently
