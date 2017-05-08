use super::basic_structures::{EventsSender, LocalDate};

use hyper;
use hyper_rustls;
use yup_oauth2;
use yup_oauth2::GetToken;
use serde::Serialize;
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalendarDate {
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WfhSingleDay {
    summary: String,
    start: CalendarDate,
    end: CalendarDate,
}

impl WfhSingleDay {
    fn new(name: &str, date: &LocalDate) -> Self {
        let date = date.format("%Y-%m-%d").to_string();
        let start = CalendarDate { date };
        let end = start.clone();

        Self {
            summary: format!("WFH {}", name),
            start,
            end,
        }
    }
}

pub struct CalendarEventsSender {
    accessor: yup_oauth2::ServiceAccountAccess<hyper::Client>,
    http_client: hyper::Client,
}

impl CalendarEventsSender {
    pub fn new() -> Self {
        let accessor = {
            let secret = yup_oauth2::service_account_key_from_file(&"Big Brother-7850ee837610.json"
                                                                        .to_string())
                    .unwrap();

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
        Self {
            accessor,
            http_client,
        }
    }

    fn post_event<T>(&mut self, event: &T)
        where T: Serialize
    {
        let token = self.accessor
            .token(&["https://www.googleapis.com/auth/calendar"])
            .expect("Failed to get auth token");
        let event_string = serde_json::to_string(&event).unwrap();
        let res = self.http_client.post("https://www.googleapis.com/calendar/v3/calendars/fl3daetfrb0ralamlb2hau9q80%40group.calendar.google.com/events?alt=json")
                  .header(hyper::header::ContentType(mime!(Application/Json)))
                  .header(hyper::header::Authorization(hyper::header::Bearer{token: token.access_token}))
                  .body(event_string.as_str()).send();

        println!("Post calendar event result: {:?}", res);
    }
}

impl EventsSender for CalendarEventsSender {
    fn post_wfh(&mut self, name: &str, date: &LocalDate) {
        let wfh = WfhSingleDay::new(name, date);
        self.post_event(&wfh);
    }
}
