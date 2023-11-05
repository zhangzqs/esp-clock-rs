use anyhow::Result;
use http::{Request, Response};

pub struct HttpClient {
    client: reqwest::blocking::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl http_client_common::Client for HttpClient {
    fn send(&self, req: Request<&[u8]>) -> Result<Response<Vec<u8>>> {
        let (parts, body_u8) = req.into_parts();
        let mut req = reqwest::blocking::Request::new(
            parts.method,
            parts.uri.to_string().parse::<reqwest::Url>()?,
        );
        *req.headers_mut() = parts.headers;
        *req.body_mut() = Some(body_u8.to_vec().into());
        let res = self.client.execute(req)?;
        let builder = Response::builder().status(res.status());
        let builder = res
            .headers()
            .iter()
            .fold(builder, |builder, (k, v)| builder.header(k, v));
        builder.body(res.bytes()?.to_vec()).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_client_common::Client;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[test]
    fn test_client() {
        let server = MockServer::start();

        // Create a mock on the server.
        let hello_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/translate")
                .query_param("word", "hello");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body("Привет");
        });

        let client = HttpClient::new();
        let empty_slice: &[u8] = &[];
        // Send an HTTP request to the mock server. This simulates your code.
        let response = client
            .send(
                Request::builder()
                    .uri(server.url("/translate?word=hello"))
                    .body(empty_slice)
                    .unwrap(),
            )
            .unwrap();

        // Ensure the specified mock was called exactly one time (or fail with a detailed error description).
        hello_mock.assert();

        // Ensure the mock server did respond as specified.
        assert_eq!(response.status(), 200);
    }
}
