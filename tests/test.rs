use light_magic::{db, join};

db! {
    Table<User: PartialEq> => { id: usize, name: String, kind: String },
    Table<Permission> => { user_name: String, level: Level },
    Table<Criminal> => { user_name: String, entry: String  },
    None<Settings: PartialEq> => { time: usize, password: String }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
enum Level {
    #[default]
    Admin,
}

#[test]
fn normal_ops_in_persistence() {
    let db = Database::open("./tests/test.json");

    // normal adding
    db.write().user.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    assert!(db.read().user.get(&0).is_some());
    assert!(db.read().user.search("0").len() == 1);

    db.write().permission.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    assert!(db.read().permission.get(&String::from("Nils")).is_some());
    assert!(db.read().permission.search("Admin").len() == 1);

    db.write().criminal.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    assert!(db.read().criminal.get(&String::from("Nils")).is_some());
    assert!(db.read().criminal.search("No records").len() == 1);

    let settings = Settings {
        time: 1718744090,
        password: String::from("password"),
    };

    db.write().settings = settings.clone();

    assert_eq!(db.read().settings, settings);

    // editing
    db.write().criminal.edit(
        &String::from("Nils"),
        Criminal {
            user_name: String::from("Nils W."),
            entry: String::from("Stole a hot dog!"),
        },
    );

    assert!(db.read().criminal.get(&String::from("Nils")).is_none());
    assert!(db.read().criminal.get(&String::from("Nils W.")).is_some());
    assert!(db.read().criminal.search("hot dog").len() == 1);

    // deleting
    db.write().criminal.delete(&String::from("Nils W."));

    assert!(db.read().criminal.get(&String::from("Nils W.")).is_none());
}

#[test]
fn normal_ops_in_memory() {
    let db = Database::open_in_memory();

    // normal adding
    db.write().user.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    assert!(db.read().user.get(&0).is_some());
    assert!(db.read().user.search("0").len() == 1);

    db.write().permission.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    assert!(db.read().permission.get(&String::from("Nils")).is_some());
    assert!(db.read().permission.search("Admin").len() == 1);

    db.write().criminal.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    assert!(db.read().criminal.get(&String::from("Nils")).is_some());
    assert!(db.read().criminal.search("No records").len() == 1);

    // editing
    db.write().criminal.edit(
        &String::from("Nils"),
        Criminal {
            user_name: String::from("Nils W."),
            entry: String::from("Stole a hot dog!"),
        },
    );

    assert!(db.read().criminal.get(&String::from("Nils")).is_none());
    assert!(db.read().criminal.get(&String::from("Nils W.")).is_some());
    assert!(db.read().criminal.search("hot dog").len() == 1);

    // deleting
    db.write().criminal.delete(&String::from("Nils W."));

    assert!(db.read().criminal.get(&String::from("Nils W.")).is_none());
}

#[test]
fn custom_derives() {
    let db = Database::open_in_memory();

    let user1 = User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    };
    let user2 = User {
        id: 1,
        name: String::from("Alice"),
        kind: String::from("Old"),
    };

    let inserted_user1 = db.write().user.add(user1.clone()).unwrap_or(user1);
    let inserted_user2 = db.write().user.add(user2.clone()).unwrap_or(user2);

    // Ensure the inserted users are not the same (since their IDs should differ)
    assert_ne!(inserted_user1, inserted_user2);

    // Ensure the inserted users have the expected properties
    assert_eq!(inserted_user1.name, "Nils");
    assert_eq!(inserted_user2.name, "Alice");
}

#[test]
fn joins() {
    let db = Database::open_in_memory();

    // remove any if there
    db.write().permission.delete(&"Nils".to_string());
    db.write().criminal.delete(&"Nils".to_string());

    // add smth
    db.write().user.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });

    let joined =
        join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    assert!(joined.is_empty());

    // add more
    db.write().permission.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });

    db.write().criminal.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });

    let joined =
        join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    assert!(!joined.is_empty());

    // add even more
    for i in 0..4 {
        db.write().user.add(User {
            id: i,
            name: String::from("Smth".to_string() + &i.to_string()),
            kind: String::from("Young"),
        });

        db.write().permission.add(Permission {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            level: Level::Admin,
        });

        db.write().criminal.add(Criminal {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            entry: String::from("No records until this day! Keep ur eyes pealed!"),
        });
    }

    let joined =
        join!(db.read(), "Smth2", user => name, permission => user_name, criminal => user_name);

    assert!(joined.len() == 1);
    assert!(joined[0].0.name == "Smth2");
}
