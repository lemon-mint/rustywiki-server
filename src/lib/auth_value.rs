/// 인증 값 전달용 객체

#[derive(std::fmt::Debug)]
pub struct AuthValue {
    pub authorized: bool,
    pub user_id: i64,
    pub user_type: String,
}

impl AuthValue {
    pub fn is_authorized(&self) -> bool {
        self.authorized
    }

    pub fn set_values(&mut self, authorized: bool, user_id: i64, user_type: String) {
        self.authorized = authorized;
        self.user_id = user_id;
        self.user_type = user_type;
    }
}

impl AuthValue {
    pub fn new() -> AuthValue {
        AuthValue {
            authorized: false,
            user_id: -1,
            user_type: "NO".into(),
        }
    }
}
