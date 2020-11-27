use std::fs::read;
use std::net::{TcpListener, Shutdown};
use std::io::Read;

mod packet;
mod datareader;

const BUFFER_SIZE: usize = 512;

fn main() {
    let server = TcpListener::bind("127.0.0.1:25565").unwrap();

    loop {
        println!("Esperando conexões");
        let mut client = server.accept().unwrap().0;

        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        for _ in 0..5 {
            if client.read(&mut buf).unwrap() == 0 {
                break;
            }

            let packet = packet::Packet { id: buf[1], data: array_copy(&buf, 2, (buf[0] + 1) as usize).unwrap()};
            let mut reader = datareader::new(&packet.data);

            println!("{}", reader.read_varint().unwrap());
            println!("{:?}", packet.data);
        }

        client.shutdown(Shutdown::Both);
    }
}

fn array_copy(src: &[u8], start_pos: usize, end_pos: usize) -> Option<Vec<u8>> {
    let mut dest = Vec::with_capacity(end_pos - start_pos);

    for x in start_pos..end_pos {
        dest[x - start_pos] = src[x];
    }

    Some(dest)
}
