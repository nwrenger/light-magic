# light-magic

[![crates.io](https://img.shields.io/crates/v/light-magic.svg)](https://crates.io/crates/light-magic)
[![crates.io](https://img.shields.io/crates/d/light-magic.svg)](https://crates.io/crates/light-magic)
[![docs.rs](https://docs.rs/light-magic/badge.svg)](https://docs.rs/light-magic)

A lightweight, fast and easy-to-use implementation of a `persistent in-memory database`.

## Features

Please note that this database is highly optimized for read operations. Writing to the database is relatively slow when using `open` because each write operation involves writing data to the disk. These writes are done atomically, ensuring no data loss on a system-wide crash.

- **Persistent Data Storage**: Data can be saved automatically and persistently to a formatted `JSON` file via `open`, or it can be operated in-memory using `open_in_memory`.
- **Easy Table Markup**: The `db!` macro allows for straightforward table markup.
- **Powerful Data Access Functions**: Utilize functions like `search` and the `join!` macro for efficient data searching and joining.
- **Efficient Storage**: The database employs a custom `Table` data type, which uses the `BTreeMap` type from `std::collections` under the hood, for efficient storage and easy access of its tables.
- **Parallel Access Support**: Access the database in parallel using `Arc<AtomicDatabase<_>>`.

> ...and more. Look into [Todos](#todos) for more planned features!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
light_magic = "0.5.4"
```

## Examples

Using it in an `axum` Server? Look here: [maud-magic-rs](https://github.com/nwrenger/maud-magic-rs). Otherwise, look at this general example:

```rust
use light_magic::{db, join};


db! {
    Table<User> => { id: usize, name: String, kind: String },
    Table<Permission> => { user_name: String, level: Level },
    Table<Criminal> => { user_name: String, entry: String  },
    Custom<Settings> => { time: usize, password: String }
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
