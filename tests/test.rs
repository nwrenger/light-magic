use light_magic::db;

db! {
    user => { id: usize, name: String, kind: String },
    permission => { user_name: String, level: Level },
    criminal => { user_name: String, entry: String }
}

#[derive(Debug)]
enum Level {
    Admin,
}

#[test]
fn test() {
    let mut db = Database::new();
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
