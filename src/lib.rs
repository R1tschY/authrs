#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use] extern crate diesel;
extern crate rocket;
extern crate dotenv;

pub mod schema;
pub mod models;
pub mod db_connection;