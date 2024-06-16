use light_magic::{db, join};

db! {
    user: [PartialEq] => { id: usize, name: String, kind: String },
    permission => { user_name: String, level: Level },
    criminal => { user_name: String, entry: String  }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
enum Level {
    #[default]
    Admin,
}

#[test]
fn normal_ops() {
    let db = Database::new(Path::new("./tests/test1.json"));

    // normal adding
    db.write().add_user(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    assert!(db.read().get_user(&0).is_some());
    assert!(db.read().search_user("0").len() == 1);

    db.write().add_permission(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    assert!(db.read().get_permission(&String::from("Nils")).is_some());
    assert!(db.read().search_permission("Admin").len() == 1);

    db.write().add_criminal(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    assert!(db.read().get_criminal(&String::from("Nils")).is_some());
    assert!(db.read().search_criminal("No records").len() == 1);

    // editing
    db.write().edit_criminal(
        &String::from("Nils"),
        Criminal {
            user_name: String::from("Nils W."),
            entry: String::from("Stole a hot dog!"),
        },
    );

    assert!(db.read().get_criminal(&String::from("Nils")).is_none());
    assert!(db.read().get_criminal(&String::from("Nils W.")).is_some());
    assert!(db.read().search_criminal("hot dog").len() == 1);

    // deleting
    db.write().delete_criminal(&String::from("Nils W."));

    assert!(db.read().get_criminal(&String::from("Nils W.")).is_none());
}

#[test]
fn custom_derives() {
    let db = Database::new(Path::new("./tests/test2.json"));

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

    let inserted_user1 = db.write().add_user(user1.clone()).unwrap_or(user1);
    let inserted_user2 = db.write().add_user(user2.clone()).unwrap_or(user2);

    // Ensure the inserted users are not the same (since their IDs should differ)
    assert_ne!(inserted_user1, inserted_user2);

    // Ensure the inserted users have the expected properties
    assert_eq!(inserted_user1.name, "Nils");
    assert_eq!(inserted_user2.name, "Alice");
}

#[test]
fn joins() {
    let db = Database::new(Path::new("./tests/test3.json"));

    // remove any if there
    db.write().delete_permission(&"Nils".to_string());
    db.write().delete_criminal(&"Nils".to_string());

    // add smth
    db.write().add_user(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });

    let joined =
        join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    assert!(joined.is_empty());

    // add more
    db.write().add_permission(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });

    db.write().add_criminal(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });

    let joined =
        join!(db.read(), "Nils", user => name, permission => user_name, criminal => user_name);
    assert!(!joined.is_empty());

    // add even more
    for i in 0..4 {
        db.write().add_user(User {
            id: i,
            name: String::from("Smth".to_string() + &i.to_string()),
            kind: String::from("Young"),
        });

        db.write().add_permission(Permission {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            level: Level::Admin,
        });

        db.write().add_criminal(Criminal {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            entry: String::from("No records until this day! Keep ur eyes pealed!"),
        });
    }

    let joined =
        join!(db.read(), "Smth2", user => name, permission => user_name, criminal => user_name);

    assert!(joined.len() == 1);
    assert!(joined[0].0.name == "Smth2");
}
