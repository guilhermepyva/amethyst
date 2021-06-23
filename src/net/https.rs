use openssl::ssl::{SslConnector, SslMethod};
use std::net::TcpStream;
use std::io::{Write, Read};

pub struct HttpsConnection {

}

pub fn test() {
    let connector = SslConnector::builder(SslMethod::tls_client()).unwrap().build();

    let mut stream = connector.connect(
        "sessionserver.mojang.com",
        TcpStream::connect("sessionserver.mojang.com:443").unwrap()
    ).unwrap();

    let get = b"GET /session/minecraft/profile/0ecda3389eeb413e962958bd0d552e5e HTTP/1.0\
    \nHost: sessionserver.mojang.com\
    \nConnection: keep-alive\
    \nDate: Tue, 02 Mar 2021 20:39:17
    \r\n\r\n";
    stream.write_all(get).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    println!("{}", String::from_utf8_lossy(&res));
}