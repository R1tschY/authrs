
pub enum AuthMethod {
    UserPassword,

    OAuth(Box<OAuth2>)
}

pub trait OAuth2 {
    fn login_url(config: AuthConfig) -> String;
    fn name() -> &'static str;
}

