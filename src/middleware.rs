use std::{collections::HashSet, io::Write};

use crate::{handler::Handler, request::Request, response::Response};
use flate2::{write::GzEncoder, Compression};

pub type Middleware = dyn FnOnce(Box<Handler>) -> Box<Handler>;

pub fn gzip_compress_middleware(handler: Box<Handler>) -> Box<Handler> {
    const ACCEPT_ENCODING_HEADER: &str = "Accept-Encoding";
    const CONTENT_ENCODING_HEADER: &str = "Content-Encoding";
    const GZIP_ENCODING: &str = "gzip";

    let compress = |mut response: Response| -> Result<Response, std::io::Error> {
        response.headers.insert(
            CONTENT_ENCODING_HEADER.to_string(),
            GZIP_ENCODING.to_string(),
        );

        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&response.body)?;
        let compressed_body = e.finish()?;

        let content_type = response
            .headers
            .get("Content-Type".to_lowercase().as_str())
            .map_or(String::new(), |s| s.to_owned());

        Ok(response.with_body(&content_type, compressed_body))
    };

    let wrapped = move |request: &Request| -> Result<Response, std::io::Error> {
        let accept_encodings = request
            .headers
            .get(ACCEPT_ENCODING_HEADER.to_lowercase().as_str())
            .map(|s| s.split(',').map(|s| s.trim()).collect::<HashSet<_>>())
            .unwrap_or_default();

        let response_result = handler(request);
        match response_result {
            Ok(response) => {
                if accept_encodings.contains(GZIP_ENCODING) {
                    compress(response)
                } else {
                    Ok(response)
                }
            }
            Err(e) => Err(e),
        }
    };

    Box::new(wrapped)
}
