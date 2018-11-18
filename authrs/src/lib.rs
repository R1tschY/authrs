#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use] extern crate diesel;
extern crate rocket;
extern crate dotenv;

pub struct UserId(pub i32);

pub mod db_connection;
pub mod schema;