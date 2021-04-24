#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use mongodb::{Client, options::ClientOptions};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::stream::StreamExt;
use r2d2_mongodb::mongodb::{
    Bson, doc, bson,
};
use mongodb::error::Error;
use r2d2::PooledConnection;
use r2d2_mongodb::{ConnectionOptions, MongodbConnectionManager};
use r2d2_mongodb::mongodb::db::ThreadedDatabase;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::env;
use std::ops::Deref;
use rocket_contrib::templates::Template;
use rocket_contrib::templates::tera::Context;
use serde::Serialize;
use rocket_contrib::json::Json;
use std::num::NonZeroUsize;


type Pool = r2d2::Pool<MongodbConnectionManager>;

pub struct Conn(pub PooledConnection<MongodbConnectionManager>);

/*
    create a connection pool of mongodb connections to allow a lot of users to modify db at same time.
*/
pub fn init_pool() -> Pool {
    dotenv().ok();
    let mongo_addr = env::var("MONGO_ADDR").unwrap_or(String::from("localhost"));
    let mongo_port = env::var("MONGO_PORT").unwrap_or(String::from("27017"));
    let db_name = env::var("DB_NAME").unwrap_or(String::from("boood"));
    let manager = MongodbConnectionManager::new(
        ConnectionOptions::builder()
            .with_host(&mongo_addr, mongo_port.parse::<u16>().unwrap())
            .with_db(&db_name)
            //.with_auth("root", "password")
            .build(),
    );
    match Pool::builder().max_size(64).build(manager) {
        Ok(pool) => pool,
        Err(e) => panic!("Error: failed to create mongodb pool {}", e),
    }
}

/*
    Create a implementation of FromRequest so Conn can be provided at every api endpoint
*/
impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Conn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(db) => Outcome::Success(Conn(db)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

/*
    When Conn is dereferencd, return the mongo connection.
*/
impl Deref for Conn {
    type Target = PooledConnection<MongodbConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct HitCount {
    count: AtomicUsize
}


#[derive(Serialize)]
struct Product {
    title:String,
    url:String,
    about:String,
    price:String,
}

#[get("/?<q>&<max>")]
fn index(q: Option<String>,max:Option<NonZeroUsize>,connection: Conn) -> Json<Vec<Product>> {
    let coll = connection.0.collection("products.amazon");

    // coll.insert_one(doc!{ "title": "Back to the Future" }, None).unwrap();
    // coll.update_one(doc!{}, doc!{ "director": "Robert Zemeckis" }, None).unwrap();
    // coll.delete_many(doc!{}, None).unwrap();
    let max = max.map_or(10usize,|x|x.get());
    let filter = q.map_or(doc! { "title": doc! {"$ne": "?"} },|q|doc! { "$text": doc! {"$search": q} });
    let mut cursor = coll.find(Some(filter), None).unwrap();
    let mut vec = vec![];
    for result in cursor.take(max) {
        if let Ok(item) = result {
            let title = String::from(item.get("title").and_then(Bson::as_str).unwrap_or("???"));
            let url = String::from(item.get("url").and_then(Bson::as_str).unwrap_or("???"));
            let price = String::from(item.get("price").and_then(Bson::as_str).unwrap_or("???"));
            let about = String::from(item.get("about").and_then(Bson::as_str).unwrap_or("???"));
            vec.push(Product{ title, price, about, url });
        }
    }
    // let mut context = Context::new();
    // context.insert("title","hello tera");
    // context.insert("body","eyy");
    // Template::render("index", &context)
    Json(vec)
}

#[get("/?<q>")]              // <- route attribute
fn world(q: String, hit_count: State<HitCount>) -> String {  // <- request handler
    let u = hit_count.count.fetch_add(1, Ordering::Relaxed);
    String::from("hello, world!") + &q + " " + &String::from(u.to_string())
}


fn main() {
    rocket::ignite()
        .manage(HitCount { count: AtomicUsize::new(0) })
        .manage(init_pool())
        .mount("/", routes![index])
        .mount("/search", routes![world])
        .attach(Template::fairing())
        .launch();
}