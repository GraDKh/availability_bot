use hyper;
use hyper_rustls;
use url;

pub fn get_response_body(name: &str) -> String {
    let mut result = String::new();
    {
        let mut serializer = url::form_urlencoded::Serializer::new(&mut result);
        serializer.append_pair("entry.656124513", name);
        serializer.append_pair("entry.1251169846", "Work from home");
        serializer.append_pair("entry.1251169846.other_option_response", "");
        serializer.append_pair("pageHistory", "0,1");
        serializer.append_pair("fvv", "1");
    }

    result
}

pub fn post_wfh(name: &str) {
    let ssl = hyper_rustls::TlsClient::new();
    let connector = hyper::net::HttpsConnector::new(ssl);


    let client = hyper::Client::with_connector(connector);
    let body = get_response_body(name);
    println!("Body is {:?}", body);

    let res = client.post("https://docs.google.com/forms/d/e/1FAIpQLSdlGgW3SXpz0SCRsSDFTDJRG4feuC9Ge1-3AdJuHUWZmOeKvg/formResponse")
            .header(hyper::header::ContentType(mime!(Application/WwwFormUrlEncoded)))
            .body(body.as_str())
            .send();
    match res {
        Ok(_) => {},
        Err(err) => println!("http post failed {:?}", err),
    };
}