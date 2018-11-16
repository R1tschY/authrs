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
    code: String
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
    cookie_name: Cow<'static, str>,
}

impl Environment {
    pub fn new(config: &Config) -> Environment {
        Environment {
            gh_client_id: Environment::get_var("GITHUB_CLIENT_ID"),
            gh_secret_id: Environment::get_var("GITHUB_SECRET_ID"),
            domain: Environment::get_var("DOMAIN"),
            uses_https: !config.environment.is_dev(),
            login_time: Duration::days(365),
            cookie_name: Cow::from("auth_token"),
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
    pub fn cookie_name(&self) -> Cow<'static, str> { self.cookie_name.clone() }
}

fn create_auth_cookie<T>(value: T, env: &Environment) -> Cookie<'static>
    where T: Into<Cow<'static, str>> {

    Cookie::build(env.cookie_name(), value)
        .path("/")
        .http_only(true)
        //.same_site(SameSite::Strict)
        .secure(env.uses_https())
        .max_age(env.login_time())
        .finish()
}

fn auth_cookie_ref(env: &Environment) -> Cookie<'static> {
    Cookie::build(env.cookie_name(), "")
        .path("/")
        .finish()
}


#[get("/")]
fn index(mut cookies: Cookies, db: DbConn, env: State<Environment>) -> String {
    // TODO: redirect to github if not logged in else redirect to callback

    let maybe_uid = if let Some(auth_token) = cookies.get("auth_token") {
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
            cookies.remove(auth_cookie_ref(env.inner()));
            None
        }
    } else {
        None
    };

    if let Some(uid) = maybe_uid {
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
        format!(
            "Login: https://github.com/login/oauth/authorize?client_id={}",
            env.github_client_id()
        )
    }
}

#[get("/callback?<code>")]
fn callback(
    mut cookies: Cookies,
    code: SessionCode,
    db: DbConn,
    env: State<Environment>
) -> Redirect {
    let client = reqwest::Client::new();

    // TODO: handle 401, 403
    let res: AccessToken = client.post("https://github.com/login/oauth/access_token")
        .form(&[
            ("client_id", env.github_client_id()),
            ("client_secret", env.github_secret_id()),
            ("code", &code.code),
        ])
        .header(ACCEPT, "application/json")
        .send()
        .unwrap()
        .json()
        .unwrap();

    let user_info: serde_json::Value = client
        .get("https://api.github.com/user")
        .header(ACCEPT, "Accept: application/vnd.github.v3+json")
        .header(AUTHORIZATION, format!("token {}", &res.access_token))
        .send()
        .unwrap()
        .json()
        .unwrap();

    let login_name = user_info.get("login").unwrap().as_str().unwrap();

    let maybe_user_id: QueryResult<i32> = {
        use authrs::schema::users::dsl::*;
        users
            .filter(login.eq(login_name))
            .filter(login_provider.eq("github"))
            .select(id)
            .first::<i32>(&db.0)
    };

    let user_id_ = maybe_user_id.unwrap_or_else(|err|{ // TODO: check Err(NotFound)
        // register user
        use authrs::schema::users::dsl::*;

        let new_user = (login_provider.eq("github"), login.eq(login_name), name.eq(login_name));
        insert_into(users).values(&new_user).execute(&db.0).unwrap();

        users
            .filter(login.eq(login_name))
            .filter(login_provider.eq("github"))
            .select(id)
            .first::<i32>(&db.0)
            .unwrap()
    });

    {
        use authrs::schema::sessions::dsl::*;

        let new_session = (token.eq(&res.access_token), expires.eq(0), user_id.eq(user_id_)); // TODO: expires
        insert_into(sessions).values(&new_session).execute(&db.0).unwrap();
    }

    cookies.add(create_auth_cookie(res.access_token, env.inner()));

    Redirect::to("/auth/")  // TODO: create callback protocol for caller
}

fn main() {
    dotenv().ok();

    rocket::ignite()
        .manage(init_db_contection_pool())
        .attach(AdHoc::on_attach(|rocket| {
            let env = Environment::new(rocket.config());
            Ok(rocket.manage(env))
        }))
        .mount("/auth", routes![index, callback])
        .launch();
}