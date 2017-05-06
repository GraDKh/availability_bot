use super::basic_structures::EventsSender;

use hyper;
use hyper_rustls;
use chrono;
use yup_oauth2;
use yup_oauth2::GetToken;

pub struct CalendarEventsSender {
    accessor: yup_oauth2::ServiceAccountAccess<hyper::Client>,
    http_client: hyper::Client,
}

impl CalendarEventsSender {
    pub fn new() -> Self {
        let accessor = {
                let secret = yup_oauth2::service_account_key_from_file(&"Big Brother-7850ee837610.json".to_string()).unwrap();
    
                let ssl = hyper_rustls::TlsClient::new();
                let connector = hyper::net::HttpsConnector::new(ssl);
                let client = hyper::Client::with_connector(connector);
                
                yup_oauth2::ServiceAccountAccess::new(secret, client)
        };
        let http_client = {
            let ssl = hyper_rustls::TlsClient::new();
            let connector = hyper::net::HttpsConnector::new(ssl);
            hyper::Client::with_connector(connector)
        };
        Self {accessor, http_client}
    }
}

impl EventsSender for CalendarEventsSender {
    fn post_wfh(&mut self,
                name: &str,
                date: &chrono::Date<chrono::Local>) {
        let token = self.accessor
                    .token(&["https://www.googleapis.com/auth/calendar"])
                    .expect("Failed to create CalendarEventsSender");
        let res = self.http_client.post("https://www.googleapis.com/calendar/v3/calendars/fl3daetfrb0ralamlb2hau9q80%40group.calendar.google.com/events?alt=json")
                  .header(hyper::header::ContentType(mime!(Application/Json)))
                  .header(hyper::header::Authorization(hyper::header::Bearer{token: token.access_token}))
                  .body("{summary: \"Test event from bot\", start: {date: \"2017-05-05\"}, end: {date: \"2017-05-05\"}}").send();

        println!("Post calendar event result: {:?}", res);
    }
}