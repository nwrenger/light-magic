# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight and easy-to-use implementation of an `in-memory database`.

## Features

- This crate utilizes the `BTreeMap` from `std::collections` for storing and accessing it's data.
- Easy markup of tables using the `db!` macro
- Useful data accessing functions like `search` to search the data

...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.1.0"
```

## Example

```rs
use light_magic::db;

db! {
    user => { id: usize, name: String, kind: String },
    permission => { user_name: String, level: Level },
    criminal => { user_name: String, entry: String }
}

#[derive(Debug, Clone)]
enum Level {
    Admin,
}

fn main() {
    let db = Database::new();
    db.insert_user(User {
        id: 0,
        name: "Nils".to_owned(),
        kind: "Young".to_owned(),
    });
    println!("{:?}", db.get_user(&0));
    println!("{:?}", db.search_user("0"));

    db.insert_permission(Permission {
        user_name: "Nils".to_owned(),
        level: Level::Admin,
    });
    println!("{:?}", db.get_permission(&String::from("Nils")));
    println!("{:?}", db.search_permission("Admin"));

    db.insert_criminal(Criminal {
        user_name: "Nils".to_owned(),
        entry: "No records until this day! Keep ur eyes pealed!".to_owned(),
    });
    println!("{:?}", db.get_criminal(&String::from("Nils")));
    println!("{:?}", db.search_criminal("No records"));
}
```

## Todos

- [ ] Saving data to storage so make the data persistent
- [ ] Add `join` or some kind of joining data together
- [x] Add parallelizing
- [x] Add Search
- [x] Add Getting, Inserting, Deleting
- [x] `db!` macro for structure of the Database
