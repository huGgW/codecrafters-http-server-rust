use crate::{handler::Handler, request::Request, response::Response};

pub type Middleware = dyn FnOnce(Box<Handler>) -> Box<Handler>;

pub fn gzip_compress_middleware(handler: Box<Handler>) -> Box<Handler> {
    const ACCEPT_ENCODING_HEADER: &str = "Accept-Encoding";
    const CONTENT_ENCODING_HEADER: &str = "Content-Encoding";
    const GZIP_ENCODING: &str = "gzip";

    let wrapped = move |request: &Request| -> Result<Response, std::io::Error> {
        let accept_encoding = request
            .headers
            .get(ACCEPT_ENCODING_HEADER.to_lowercase().as_str());

        let response_result = handler(request);
        match response_result {
            Ok(mut response) => match accept_encoding {
                Some(&ref e) if e == GZIP_ENCODING => {
                    _ = response.headers.insert(
                        CONTENT_ENCODING_HEADER.to_string(),
                        GZIP_ENCODING.to_string(),
                    );
                    Ok(response)
                }
                _ => Ok(response),
            },
            Err(e) => Err(e),
        }
    };

    Box::new(wrapped)
}
