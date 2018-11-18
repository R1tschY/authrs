#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate dotenv;
extern crate reqwest;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate time;
extern crate cookie;
extern crate serde_json;

use std::env;
use diesel::prelude::*;

use dotenv::dotenv;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use time::Duration;
use std::ops::Add;

use authrs::{
    db_connection::DbConn,
    db_connection::init_db_contection_pool
};
use diesel::insert_into;

use cookie::SameSite;
use rocket::{
    fairing::AdHoc,
    http::Cookie,
    http::Cookies,
    State,
    Config
};
use std::borrow::Cow;
use rocket::response::Redirect;

#[derive(FromForm)]
struct SessionCode {
    code: String,
    state: String,
}

#[derive(FromForm)]
struct LoginParams {
    // callback website
    callback: String,
}

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
    scope: String,
    token_type: String,
}

struct Environment {
    gh_client_id: String,
    gh_secret_id: String,
    domain: String,
    uses_https: bool,
    login_time: Duration,
    token_cookie: Cow<'static, str>,
    state_cookie: Cow<'static, str>,
    continue_cookie: Cow<'static, str>,
}

impl Environment {
    pub fn new(config: &Config) -> Environment {
        Environment {
            gh_client_id: Environment::get_var("GITHUB_CLIENT_ID"),
            gh_secret_id: Environment::get_var("GITHUB_SECRET_ID"),
            domain: Environment::get_var("DOMAIN"),
            uses_https: !config.environment.is_dev(),
            login_time: Duration::days(365),
            token_cookie: Cow::from("auth_token"),
            state_cookie: Cow::from("auth_state"),
            continue_cookie: Cow::from("auth_continue"),
        }
    }

    fn get_var(name: &str) -> String {
        env::var(name).expect(&format!("{} must be set", name))
    }

    pub fn github_client_id(&self) -> &str { &self.gh_client_id }
    pub fn github_secret_id(&self) -> &str { &self.gh_secret_id }
    pub fn domain(&self) -> &str { &self.domain }
    pub fn uses_https(&self) -> bool { self.uses_https }
    pub fn login_time(&self) -> Duration { self.login_time }
    pub fn token_cookie(&self) -> Cow<'static, str> { self.token_cookie.clone() }
    pub fn state_cookie(&self) -> Cow<'static, str> { self.state_cookie.clone() }
}

fn create_cookie<T>(name: Cow<'static, str>, value: T, env: &Environment) -> Cookie<'static>
    where T: Into<Cow<'static, str>> {

    Cookie::build(name, value)
        .path("/")
        .http_only(true)
        //.same_site(SameSite::Strict)
        .secure(env.uses_https())
        .max_age(env.login_time()) // TODO: use expire time of token
        .finish()
}

fn cookie_ref(name: Cow<'static, str>, env: &Environment) -> Cookie<'static> {
    Cookie::build(name, "")
        .path("/")
        .finish()
}

struct User(i32);

fn auth_user(mut cookies: Cookies, db: DbConn, env: State<Environment>) -> Option<User> {
    if let Some(auth_token) = cookies.get(env.token_cookie()) {
        use authrs::schema::sessions::dsl::*;

        let maybe_session_info = sessions
            .find(auth_token.value())
            .select((user_id, expires))
            .first::<(i32, i64)>(&db.0)
            .optional()
            .unwrap();
        if let Some(session_info) = maybe_session_info {
            let (uid, exps) = session_info;
            // TODO: check expires
            Some(uid)
        } else {
            cookies.remove(cookie_ref(env.token_cookie, env.inner()));
            None
        }
    } else {
        None
    }
}

pub fn login_user(provider_name: &str, login_name: &str) -> Option<UserId> {
    let maybe_user_id: QueryResult<i32> = {
        use authrs::schema::users::dsl::*;
        users
            .filter(login.eq(login_name))
            .filter(login_provider.eq(provider_name))
            .select(id)
            .first::<i32>(&db.0)
    };

    if let Ok(user_id) = maybe_user_id {
        Some(UserId(user_id))
    } else if maybe_user_id == Err(NotFound) {
        None
    } else {
        None
    }
}

pub fn register_user(provider_name: &str, login_name: &str) {
    use authrs::schema::users::dsl::*;

    let new_user = (login_provider.eq(provider_name), login.eq(&login_name), name.eq(&login_name));
    insert_into(users).values(&new_user).execute(&db.0).unwrap();

    users
        .filter(login.eq(&session_info.login_name))
        .filter(login_provider.eq(provider_name))
        .select(id)
        .first::<i32>(&db.0)
        .unwrap()
}

pub fn add_session() {
    use authrs::schema::sessions::dsl::*;

    let new_session = (token.eq(&res.access_token), expires.eq(0), user_id.eq(user_id_)); // TODO: expires
    insert_into(sessions).values(&new_session).execute(&db.0).unwrap();
}

pub struct SessionInfo {
    access_token: String,
    login_name: String
}

pub mod github {
    pub fn login_form(state: &str, env: &Environment) -> String {
        format!(
            r#"<a href="https://github.com/login/oauth/authorize?client_id={}&state={}">Login with GitHub</a>"#,
            env.github_client_id(),
            state
        )
    }

    pub fn callback(code: &str) -> SessionInfo {
        let client = reqwest::Client::new();

        // TODO: handle 401, 403
        let res: AccessToken = client.post("https://github.com/login/oauth/access_token")
            .form(&[
                ("client_id", env.github_client_id()),
                ("client_secret", env.github_secret_id()),
                ("code", &code),
            ])
            .header(ACCEPT, "application/json")
            .send()
            .unwrap()
            .json()
            .unwrap();

        let user_info: serde_json::Value = client
            .get("https://api.github.com/user")
            .header(ACCEPT, "application/vnd.github.v3+json")
            .header(AUTHORIZATION, format!("token {}", &res.access_token))
            .send()
            .unwrap()
            .json()
            .unwrap();

        let login_name = user_info.get("login").unwrap().as_str().unwrap();

        SessionInfo { access_token: res.access_token, login_name }
    }
}


#[get("/info")]
fn info(mut cookies: Cookies, db: DbConn, env: State<Environment>) -> String {
    // TODO: redirect to github if not logged in else redirect to callback
    if let Some(User(uid)) = auth_user(cookies, db, env) {
        let user_name = {
            use authrs::schema::users::dsl::*;
            users
                .filter(id.eq(uid))
                .select(name)
                .first::<String>(&db.0)
                .unwrap()
        };

        format!(
            "Hello {}!", user_name
        )
    } else {
        Redirect.to("/?callback=/info")
    }
}

#[get("/?<params>")]
fn login_form(
    mut cookies: Cookies,
    params: LoginParams,
    db: DbConn,
    env: State<Environment>
) -> Redirect/HTML {
    if let Some(uid) = auth_user(cookies, db, env) {
        Redirect.to(params.callback)  // TODO: check callback
    } else {
        let state = "ABCDEF"; // TODO: generate random key
        cookies.add(create_cookie(env.state_cookie(), state, env.inner())); // TODO: expire
        cookies.add(create_cookie(env.continue_cookie(), params.callback, env.inner())); // TODO: expire

        github::login_form(state, env.inner())
    }
}

#[get("/callback?<code>")]
fn callback(
    mut cookies: Cookies,
    code: SessionCode,
    db: DbConn,
    env: State<Environment>
) -> Redirect {
    let state = cookies.get(env.state_cookie()).unwrap();
    if state != code.state {
        panic!("CSRF detected");
    }

    let session_info = github::callback(&code.code);

    login_user();

    let user_id_ = maybe_user_id.unwrap_or_else(||{
        register_user()
    });

    add_session();

    cookies.add(create_cookie(env.token_cookie, res.access_token, env.inner()));

    let callback = cookies.get(env.callback_cookie())
    Redirect::to(callback) // TODO: validate callback
}

fn main() {
    dotenv().ok();

    rocket::ignite()
        .manage(init_db_contection_pool())
        .attach(AdHoc::on_attach(|rocket| {
            let env = Environment::new(rocket.config());
            Ok(rocket.manage(env))
        }))
        .mount("/", routes![info, login_form, callback])
        .launch();
}