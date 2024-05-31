use light_magic::db;

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
fn test() {
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
