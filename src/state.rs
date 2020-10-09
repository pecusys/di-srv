use actix::{Actor, Addr, Context, Handler};
use std::collections::HashMap;
use divdb::db::Db;
use actix_web::{self, web, HttpRequest, HttpResponse};

pub fn state() -> State {
    let db = Db::new_blocking().unwrap();
    let state = State { db };
    state
}

pub struct Config {
    db_url: String,
    api_key: Vec<u8>,
    secret_key: Vec<u8>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            db_url: String::new(),
            api_key: Vec::new(),
            secret_key: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub db: Db,
}

impl State {

    pub async fn new() -> Self {
        let db = Db::new_blocking().unwrap();
        Self { db }
    }
}

impl Actor for State { 
    type Context = Context<Self>;
}
