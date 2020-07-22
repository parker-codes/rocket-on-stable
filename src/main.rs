#[macro_use] extern crate rocket;

use rocket::State;
use std::sync::atomic::{AtomicU32, Ordering};

struct HitCount {
    value: AtomicU32,
}

#[get("/")]
fn index(hit_count: State<HitCount>) -> String {
    hit_count.value.fetch_add(1, Ordering::Relaxed);

    match hit_count.value.load(Ordering::Relaxed) {
        n @ 1..=9 => n.to_string(),
        _              => "10+".to_string(),
    }
}

#[launch]
fn rocket() -> rocket::Rocket {
    let hit_count = HitCount { value: AtomicU32::new(0) };

    rocket::ignite()
        .mount("/", routes![index])
        .manage(hit_count)
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local::blocking::Client;
    use rocket::http::Status;

    #[test]
    fn increments_on_page_hits() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("1".into()));

        // second hit
        let response = client.get("/").dispatch();
        assert_eq!(response.into_string(), Some("2".into()));
    }

    #[test]
    fn caps_at_ten() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let mut i = 0;
        let response = loop {
            let response = client.get("/").dispatch();
            if i == 10 { break response; }
            i += 1;
        };

        assert_eq!(response.into_string(), Some("10+".into()));
    }
}