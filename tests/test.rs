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
}
