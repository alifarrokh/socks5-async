lazy_static! {
    static ref USERS: Vec<User> = vec![
        User {
            username: "ali".to_string(),
            password: "123456".to_string(),
        },
        User {
            username: "admin".to_string(),
            password: "123456".to_string(),
        },
    ];
}
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    username: String,
    password: String,
}

impl User {
    pub fn new(username: String, password: String) -> User {
        User {username, password}
    }
    pub fn auth(user: &User) -> bool {
        USERS.contains(user)
    }
}