// TODO: Add static html doc to show basic routes.
#[get("/")]
pub fn home() -> &'static str {
    "Welcome to the unoffical Super Auto Pets API!"
}

#[cfg(test)]
mod test {
    use crate::server::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn test_home() {
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client.get(uri!(super::home)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some("Welcome to the unoffical Super Auto Pets API!".into())
        );
    }
}
