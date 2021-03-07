use http_lib::request::request_method::RequestMethod;
use http_lib::request::Request;
use http_lib::response::Response;

use crate::server::Server;

pub struct Handler {
    method: RequestMethod,
    handler: fn(&Request) -> Response,
}

impl Handler {
    pub fn new(method: RequestMethod, handler: fn(&Request) -> Response) -> Self {
        Handler { method, handler }
    }

    pub fn register(
        server: &mut Server,
        uri: &str,
        method: RequestMethod,
        handler: fn(&Request) -> Response,
    ) {
        let handlers = server
            .routes
            .get_mut(uri)
            .expect("Server router is missing endpoint");

        let handler = Handler::new(method, handler);

        handlers.push(handler);
    }

    pub fn run(&self, request: &Request) -> Response {
        (self.handler)(request)
    }

    pub fn method(&self) -> &RequestMethod {
        &self.method
    }
}
