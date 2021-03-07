use http_lib::request::request_method::RequestMethod;
use http_lib::request::Request;
use http_lib::response::Response;

use crate::handler::Handler;
use crate::make_handler;
use crate::server::Server;

pub struct Route<'a> {
    server: &'a mut Server,
    uri: &'a str,
}

impl<'a> Route<'a> {
    pub fn new(server: &'a mut Server, uri: &'a str) -> Self {
        if !server.routes.contains_key(uri) {
            server.routes.insert(uri.to_owned(), vec![]);
        }

        Route { server, uri }
    }

    make_handler!(get, RequestMethod::Get);
    make_handler!(head, RequestMethod::Head);
    make_handler!(post, RequestMethod::Post);
    make_handler!(put, RequestMethod::Put);
    make_handler!(delete, RequestMethod::Delete);
    make_handler!(trace, RequestMethod::Trace);
    make_handler!(options, RequestMethod::Options);
    make_handler!(connect, RequestMethod::Connect);
    make_handler!(patch, RequestMethod::Patch);
}

#[macro_export]
macro_rules! make_handler {
    ($handler_name:ident, $method:path) => {
        pub fn $handler_name(&mut self, handler: fn(&Request) -> Response) -> &mut Self {
            Handler::register(self.server, self.uri, $method, handler);
            self
        }
    };
}
