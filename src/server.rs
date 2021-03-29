use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;

use http_lib::error::TcpIpError;
use http_lib::http_item::HttpItem;
use http_lib::request::Request;
use http_lib::response::response_status::ResponseStatus;
use http_lib::response::{Response, ResponseBuilder};
use http_lib::stream_helper::setup_stream;
use http_lib::Result;

use crate::handler::Handler;
use crate::route::Route;
use crate::thread_pool::ThreadPool;

pub struct Server {
    pub address: String,
    pub timeout: u64,
    pub routes: HashMap<String, Vec<Handler>>,
    pool: ThreadPool,
}

impl Server {
    pub fn new<T: AsRef<str>>(address: T, timeout: u64) -> Self {
        let pool = ThreadPool::new(8);

        Server {
            address: address.as_ref().to_owned(),
            timeout,
            routes: HashMap::new(),
            pool,
        }
    }

    pub fn at(&mut self, uri: &'static str) -> Route {
        Route::new(self, uri)
    }

    fn send_response(writer: &mut BufWriter<&TcpStream>, response: Response) -> Result<()> {
        let response_bytes = response.to_bytes()?;
        writer.write_all(&response_bytes)?;
        writer.flush()?;

        Ok(())
    }

    pub fn start(mut self) -> Result<()> {
        let local_address = SocketAddrV4::from_str(&self.address)?;
        let timeout = self.timeout;
        let routes = Arc::new(self.routes);

        println!(
            "Server started at 'http://{}'. Timeout is {} seconds.\n",
            local_address, timeout
        );

        let local_server = TcpListener::bind(local_address)?;

        for stream in local_server.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = routes.clone();

                    self.pool.spawn(move || {
                        if let Err(e) = setup_stream(&stream, timeout) {
                            eprintln!("{}", e);
                            return;
                        }

                        let mut local_reader = BufReader::new(&stream);
                        let mut local_writer = BufWriter::new(&stream);

                        loop {
                            match Request::from_reader(&mut local_reader) {
                                Ok(request) => {
                                    if let Some(handlers) = routes.get(&request.header.uri) {
                                        let correct_handler = handlers
                                            .iter()
                                            .find(|h| *h.method() == request.header.method);

                                        if let Some(handler) = correct_handler {
                                            let response = handler.run(&request);

                                            if let Err(e) =
                                                Self::send_response(&mut local_writer, response)
                                            {
                                                eprintln!("{}", e);
                                            }
                                        } else {
                                            let response = ResponseBuilder::new()
                                                .status_code(
                                                    ResponseStatus::MethodNotAllowed as u16,
                                                )
                                                .build()
                                                .expect(
                                                    "Failed to create Method Not Allowed response.",
                                                );

                                            if let Err(e) =
                                                Self::send_response(&mut local_writer, response)
                                            {
                                                eprintln!("{}", e);
                                            }
                                        }
                                    } else {
                                        let response = ResponseBuilder::new()
                                            .status_code(ResponseStatus::NotFound as u16)
                                            .build()
                                            .expect("Failed to create Not Found response.");

                                        if let Err(e) =
                                            Self::send_response(&mut local_writer, response)
                                        {
                                            eprintln!("{}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    if e != TcpIpError::TcpTimeout && e != TcpIpError::DataTimeout {
                                        eprintln!("{}", e);
                                    }

                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }

        Ok(())
    }
}
