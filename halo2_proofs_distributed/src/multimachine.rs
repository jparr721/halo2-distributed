//! Multi
///
///
use futures::future::join_all;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::net::TcpStream;
use std::sync::Mutex;
use std::{io, net::SocketAddr, thread, time::Duration};
use stubborn_io::StubbornTcpStream;

lazy_static! {
    static ref TCP_CONNECTIONS: Mutex<Vec<TcpStream>> = {
        let connections = vec!["127.0.0.1:8081", "127.0.0.1:8082"]
            .into_iter()
            .map(|addr| TcpStream::connect(addr).unwrap())
            .collect::<Vec<_>>();
        Mutex::new(connections)
    };
}

/// Get all active TCP connections
pub fn get_connections() -> &'static Mutex<Vec<TcpStream>> {
    &TCP_CONNECTIONS
}
