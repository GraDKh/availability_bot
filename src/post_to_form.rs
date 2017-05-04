use hyper;
use hyper_rustls;
use url;
use chrono;
use chrono::Datelike;
use yup_oauth2;
use yup_oauth2::GetToken;

pub fn get_response_body(name: &str) -> String {
    let mut result = String::new();
    {
        let mut serializer = url::form_urlencoded::Serializer::new(&mut result);
        serializer.append_pair("entry.656124513", name);

        let now = chrono::prelude::Local::now();
        serializer.append_pair("entry.1765270110_year", now.year().to_string().as_str());
        serializer.append_pair("entry.1765270110_month", now.month().to_string().as_str());
        serializer.append_pair("entry.1765270110_day", now.day().to_string().as_str());

        serializer.append_pair("entry.1251169846", "Work from home");
        serializer.append_pair("entry.1251169846.other_option_response", "");
        
        serializer.append_pair("pageHistory", "0,1");
        serializer.append_pair("fvv", "1");
    }

    result
}

fn auth() {
    let secret = yup_oauth2::service_account_key_from_file(&"Big Brother-7850ee837610.json".to_string()).unwrap();
    
    let ssl = hyper_rustls::TlsClient::new();
    let connector = hyper::net::HttpsConnector::new(ssl);
    let client = hyper::Client::with_connector(connector);
    
    let res = yup_oauth2::ServiceAccountAccess::new(secret, client)
                        .token(&["https://www.googleapis.com/auth/calendar"]);
    match res {
        Ok(t) => { 
            println!("Authorised! {:?}", t);

            let ssl = hyper_rustls::TlsClient::new();
            let connector = hyper::net::HttpsConnector::new(ssl);
            let client = hyper::Client::with_connector(connector);
            let res = client.post("https://www.googleapis.com/calendar/v3/calendars/fl3daetfrb0ralamlb2hau9q80%40group.calendar.google.com/events?alt=json")
                  .header(hyper::header::ContentType(mime!(Application/Json)))
                  .header(hyper::header::Authorization(hyper::header::Bearer{token: t.access_token}))
                  .body("{summary: \"Test event cfrom bot\", start: {date: \"2017-05-05\"}, end: {date: \"2017-05-05\"}}").send();

            println!("Post calendar event result: {:?}", res);

        },
        Err(err) => println!("Failed to acquire token: {}", err),
    }
}

pub fn post_wfh(name: &str) {
    let ssl = hyper_rustls::TlsClient::new();
    let connector = hyper::net::HttpsConnector::new(ssl);


    let client = hyper::Client::with_connector(connector);
    let body = get_response_body(name);

    let res = client.post("https://docs.google.com/forms/d/e/1FAIpQLSdlGgW3SXpz0SCRsSDFTDJRG4feuC9Ge1-3AdJuHUWZmOeKvg/formResponse")
            .header(hyper::header::ContentType(mime!(Application/WwwFormUrlEncoded)))
            .body(body.as_str())
            .send();
    match res {
        Ok(_) => {},
        Err(err) => println!("http post failed {:?}", err),
    };

    auth();
}