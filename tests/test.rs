use light_magic::{db, join};

db! {
    user: [PartialEq] => { id: usize, name: &'static str, kind: &'static str },
    permission => { user_name: &'static str, level: Level },
    criminal => { user_name: &'static str, entry: &'static str  }
}

#[derive(Default, Debug, Clone)]
enum Level {
    #[default]
    Admin,
}

#[test]
fn normal_ops() {
    let db = Database::new();

    // normal inserting
    db.insert_user(User {
        id: 0,
        name: "Nils",
        kind: "Young",
    });
    assert!(db.get_user(&0).is_some());
    assert!(db.search_user("0").len() == 1);

    db.insert_permission(Permission {
        user_name: "Nils",
        level: Level::Admin,
    });
    assert!(db.get_permission(&"Nils").is_some());
    assert!(db.search_permission("Admin").len() == 1);

    db.insert_criminal(Criminal {
        user_name: "Nils",
        entry: "No records until this day! Keep ur eyes pealed!",
    });
    assert!(db.get_criminal(&"Nils").is_some());
    assert!(db.search_criminal("No records").len() == 1);

    // editing
    db.edit_criminal(
        &"Nils",
        Criminal {
            user_name: "Nils W.",
            entry: "Stole a hot dog!",
        },
    );

    assert!(db.get_criminal(&"Nils").is_none());
    assert!(db.get_criminal(&"Nils W.").is_some());
    assert!(db.search_criminal("hot dog").len() == 1);

    // deleting
    db.delete_criminal(&"Nils W.");

    assert!(db.get_criminal(&"Nils W.").is_none());
}

#[test]
fn custom_derives() {
    let db = Database::new();

    let user1 = User {
        id: 0,
        name: "Nils",
        kind: "Young",
    };
    let user2 = User {
        id: 1,
        name: "Alice",
        kind: "Old",
    };

    let inserted_user1 = db.insert_user(user1.clone()).unwrap();
    let inserted_user2 = db.insert_user(user2.clone()).unwrap();

    // Ensure the inserted users are not the same (since their IDs should differ)
    assert_ne!(inserted_user1, inserted_user2);

    // Ensure the inserted users have the expected properties
    assert_eq!(inserted_user1.name, "Nils");
    assert_eq!(inserted_user2.name, "Alice");
}

#[test]
fn joins() {
    let db = Database::new();

    // insert smth
    db.insert_user(User {
        id: 0,
        name: "Nils",
        kind: "Young",
    });

    let joined = join!(db, "Nils", user => name, permission => user_name, criminal => user_name);
    assert!(joined.0.is_some() && joined.0.unwrap_or_default().len() == 1);
    assert!(joined.1.is_none());
    assert!(joined.2.is_none());
}
