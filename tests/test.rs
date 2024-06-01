use light_magic::{db, join};

db! {
    user => { id: usize, name: &'static str, kind: &'static str },
    permission => { user_name: &'static str, level: Level },
    criminal => { user_name: &'static str, entry: &'static str  }
}

#[derive(Debug, Clone)]
enum Level {
    Admin,
}

#[test]
fn normal_ops() {
    let db = Database::new();
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
