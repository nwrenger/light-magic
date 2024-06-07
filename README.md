# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight and easy-to-use implementation of an `in-memory database`.

## Features

- This crate utilizes the `BTreeMap` from `std::collections` for storing and accessing it's data.
- Easy markup of tables using the `db!` macro
- Useful data accessing functions like `search` or `join!` macro to search the data or join data together
- Supports accessing the database in parallel using `Arc<Mutex<_>>` for each table

...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.3.2"
```

## Examples

Using it in an Axum Server? Look here: [maud-magic-rs](https://github.com/nwrenger/maud-magic-rs). Otherwise, look at this general example:

```rust
use light_magic::{db, join};

db! {
    user => { id: usize, name: &'static str, kind: &'static str },
    permission => { user_name: &'static str, level: Level },
    criminal => { user_name: &'static str, entry: &'static str  }
}

#[derive(Default, Debug, Clone)]
enum Level {
    #[default]
    Admin,
}

fn main() {
    let db = Database::new();
    db.insert_user(User {
        id: 0,
        name: "Nils",
        kind: "Young",
    });
    println!("{:?}", db.get_user(&0));
    println!("{:?}", db.search_user("0"));

    db.insert_permission(Permission {
        user_name: "Nils",
        level: Level::Admin,
    });
    println!("{:?}", db.get_permission(&"Nils"));
    println!("{:?}", db.search_permission("Admin"));

    db.insert_criminal(Criminal {
        user_name: "Nils",
        entry: "No records until this day! Keep ur eyes pealed!",
    });
    println!("{:?}", db.get_criminal(&"Nils"));
    println!("{:?}", db.search_criminal("No records"));

    let joined = join!(db, "Nils", user => name, permission => user_name, criminal => user_name);
    println!("{:?}", joined.0);
    println!("{:?}", joined.1);
    println!("{:?}", joined.2);
}
```

## Todos

- Add somehow a persistence feature -> using `Drop` or after each time changing, saving async to storage
