use cornetti::core::traits::{BaseModel, To};
use cornetti_macros::{AutoFromPartial, AutoToFull};

// Struct per test AutoFromPartial
struct PartialUser {
    name: String,
    age: i32,
    email: String,
}

#[derive(AutoFromPartial, Debug, PartialEq)]
#[auto_from(PartialUser)]
struct FullUser {
    name: String,
    age: i32,
    email: String,
}

#[test]
fn auto_from_partial_basic() {
    let partial = PartialUser {
        name: "Mario".into(),
        age: 30,
        email: "mario@test.it".into(),
    };
    let full = FullUser::from(partial);
    assert_eq!(full.name, "Mario");
    assert_eq!(full.age, 30);
    assert_eq!(full.email, "mario@test.it");
}

#[test]
fn auto_from_partial_with_strings() {
    let partial = PartialUser {
        name: "".into(),
        age: 0,
        email: "".into(),
    };
    let full = FullUser::from(partial);
    assert_eq!(full.name, "");
    assert_eq!(full.age, 0);
}

// Struct per test AutoToFull
#[derive(AutoToFull, Debug)]
#[to_full(FullModel)]
struct PartialModel {
    name: String,
    age: i32,
}

#[derive(Debug, PartialEq)]
struct FullModel {
    id: String,
    name: String,
    age: i32,
    active: bool,
    created: String,
}

impl BaseModel for FullModel {
    fn new() -> Self {
        FullModel {
            id: String::new(),
            name: String::new(),
            age: 0,
            active: true,
            created: String::new(),
        }
    }
}

#[test]
fn auto_to_full_basic() {
    let partial = PartialModel {
        name: "Luigi".into(),
        age: 25,
    };
    let full: FullModel = partial.to();
    assert_eq!(full.name, "Luigi");
    assert_eq!(full.age, 25);
    assert_eq!(full.id, "");
    assert!(full.active);
}

#[test]
fn auto_to_full_partial_overwrites_empty() {
    let partial = PartialModel {
        name: "Test".into(),
        age: 99,
    };
    let full: FullModel = partial.to();
    assert_eq!(full.name, "Test");
    assert_eq!(full.age, 99);
}

// Test con valori di default specificati via attributo #[new]
#[derive(AutoToFull, Debug)]
#[to_full(FullModelWithDefaults)]
#[new(active = false)]
struct PartialModelWithDefaults {
    name: String,
}

#[derive(Debug, PartialEq)]
struct FullModelWithDefaults {
    id: String,
    name: String,
    age: i32,
    active: bool,
    created: String,
}

impl BaseModel for FullModelWithDefaults {
    fn new() -> Self {
        FullModelWithDefaults {
            id: String::new(),
            name: String::new(),
            age: 0,
            active: true,
            created: String::new(),
        }
    }
}

#[test]
fn auto_to_full_with_defaults() {
    let partial = PartialModelWithDefaults {
        name: "DefaultUser".into(),
    };
    let full: FullModelWithDefaults = partial.to();
    assert_eq!(full.name, "DefaultUser");
    assert!(!full.active); // sovrascritto da #[new(active = false)]
}
