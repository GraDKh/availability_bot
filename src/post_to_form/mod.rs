use hyper;
use hyper_rustls;
use formdata;
use std::io;

//pub trait EventsPoster {
    pub fn post_wfh(name: &str) {
        let ssl = hyper_rustls::TlsClient::new();
        let connector = hyper::net::HttpsConnector::new(ssl);
        
        
        let client = hyper::Client::with_connector(connector);
        let form_data = formdata::FormData {
            fields: vec![("entry.656124513".to_owned(),
                         "V.Pupkin".to_owned()),
                         ("entry.1251169846".to_owned(),
                         "Work from home".to_owned()),
                         ("fvv".to_owned(),
                         "1".to_owned())],
            files : Vec::new()
        };
        let boundary = formdata::generate_boundary();
        let mut form_string = Vec::<u8>::new();
        formdata::write_formdata(&mut form_string, &boundary, &form_data);
        let form_string = String::from_utf8(form_string).unwrap();
        println!("Form data: {:?}", form_string);

        let res = client.post("https://docs.google.com/forms/d/e/1FAIpQLSdlGgW3SXpz0SCRsSDFTDJRG4feuC9Ge1-3AdJuHUWZmOeKvg/formResponse")
            .header(hyper::header::ContentType(mime!(Application/WwwFormUrlEncoded)))
            .body(form_string.as_str())
            .send();
        match res {
            Ok(result) => println!("http post succeded {:?}", result),
            Err(err) => println!("http post failed {:?}", err)
        };
    }
//}