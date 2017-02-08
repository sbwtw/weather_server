
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate geoip;
extern crate rocket;
extern crate rustc_serialize;
extern crate hyper;
extern crate hyper_native_tls;

use rocket::{Request, Route, Data};
use rocket::config::{Config, Environment};
use rocket::handler::Outcome;
use rocket::http::Method::{Get, Post};
use geoip::{GeoIp, DBType, Options};
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::sync::RwLock;
use std::collections::HashMap;
use std::io::Read;

// Powered by Dark Sky https://darksky.net/poweredby/
const API_KEY: &'static str = "91de7cb3476a2c12913fa67b90f069a7";
lazy_static!{
    static ref APP: RwLock<App> = RwLock::new(App::new());
    static ref CLIENT: Client = Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap()));
    // static ref GEOIP: Arc<Mutex<GeoIp>> = Arc::new(Mutex::new(GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap()));
}

struct App {
    cache: HashMap<String, String>,
}

impl App {
    fn new() -> App {
        App { cache: HashMap::new() }
    }

    fn cached_data<T: AsRef<str>>(&self, location: T) -> Option<&String> {
        self.cache.get(location.as_ref())
    }

    fn set_data<K: AsRef<str>, V: AsRef<str>>(&mut self, location: K, data: V) {

        self.cache.insert(location.as_ref().to_owned(), data.as_ref().to_owned());
        // self.cache.insert(location.as_ref().to_owned(), r#"{"a": 0}"#.to_owned());

        // self.cache.get(location.as_ref()).unwrap()
    }
}

fn fetch_data(latitude: f32, longitude: f32) -> String {

    let url = format!("https://api.darksky.net/forecast/{}/{},{}",
                      API_KEY,
                      latitude,
                      longitude);

    let mut response = CLIENT.get(&url).send().unwrap();
    let mut result = String::new();
    response.read_to_string(&mut result).unwrap();

    result
}

fn index(req: &Request, _: Data) -> Outcome<'static> {
    // let ip = req.remote().unwrap().ip();
    let ip = "8.8.8.8".parse().unwrap();
    // println!("{:?}", ip);
    let geoip = GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
    let info = geoip.city_info_by_ip(ip).unwrap();
    let key = format!("{},{}", info.latitude, info.longitude);

    // read cached data
    {
        let app = APP.read().unwrap();
        if let Some(v) = app.cached_data(&key) {
            return Outcome::of(v.clone());
        }
    }

    // fetch data
    let data = fetch_data(info.latitude, info.longitude);

    {
        APP.write().unwrap().set_data(key, &data);
    }

    Outcome::of(data)
}

fn main() {

    let config = Config::build(Environment::Production).address("0.0.0.0").port(8000).unwrap();

    // let geoip = GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
    // let info = geoip.city_info_by_ip("8.8.8.8".parse().unwrap()).unwrap();
    // println!("{:?}", info);

    rocket::custom(config, true)
        .mount("/",
               vec![Route::new(Get, "/", index), Route::new(Post, "/", index)])
        .launch();
}
