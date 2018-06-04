#[derive(Queryable)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub passwd: String,
}

table! {
    user_info (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        passwd -> Text,
    }
}
