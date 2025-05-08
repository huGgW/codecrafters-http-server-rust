use std::collections::HashSet;

use crate::{handler::Handler, request::Request, response::Response};

pub type Middleware = dyn FnOnce(Box<Handler>) -> Box<Handler>;

pub fn gzip_compress_middleware(handler: Box<Handler>) -> Box<Handler> {
    const ACCEPT_ENCODING_HEADER: &str = "Accept-Encoding";
    const CONTENT_ENCODING_HEADER: &str = "Content-Encoding";
    const GZIP_ENCODING: &str = "gzip";

    let wrapped = move |request: &Request| -> Result<Response, std::io::Error> {
        let accept_encodings = request
            .headers
            .get(ACCEPT_ENCODING_HEADER.to_lowercase().as_str())
            .map(|s| s.split(',').map(|s| s.trim()).collect::<HashSet<_>>())
            .unwrap_or_default();

        let response_result = handler(request);
        match response_result {
            Ok(mut response) => {
                if accept_encodings.contains(GZIP_ENCODING) {
                    response.headers.insert(
                        CONTENT_ENCODING_HEADER.to_string(),
                        GZIP_ENCODING.to_string(),
                    );
                }
                Ok(response)
            }
            Err(e) => Err(e),
        }
    };

    Box::new(wrapped)
}
