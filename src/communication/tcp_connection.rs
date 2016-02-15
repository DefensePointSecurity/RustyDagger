use std::*;
use std::io::{Write, Read};
use byteorder::{ByteOrder, LittleEndian};

pub struct TcpSession {
    pub tcp_stream: net::TcpStream,
    pub transmit_size: u32,
    pub input_buffer: Vec<u8>,
    pub output_buffer: Vec<u8>,
    pub orders: Vec<u8>,
}

impl TcpSession {
    
    fn new(tcp_stream: net::TcpStream, transmit_size: u32, input_buffer: Vec<u8>, output_buffer: Vec<u8>, orders: Vec<u8>) -> TcpSession {
        TcpSession {
            tcp_stream: tcp_stream,
            transmit_size: transmit_size,
            input_buffer: input_buffer,
            output_buffer: output_buffer,
            orders: orders,
        }
    }
    //Attempt to write all buffered data to the TcpStream
    pub fn send(&mut self) -> () {
        let mut size_buff: Vec<u8> = vec![0;4];
        LittleEndian::write_u32(&mut size_buff, self.output_buffer.len() as u32); 
        self.tcp_stream.write_all(&size_buff).unwrap();
        self.tcp_stream.write(&self.output_buffer).unwrap();
        self.output_buffer.clear();
    }

    //Attempt to read data received from the TcpStream
    pub fn recv(&mut self) -> () {
        self.input_buffer.clear();
        let mut size_buff: Vec<u8> = vec![0;4];
        self.tcp_stream.read(&mut size_buff).unwrap();
        self.transmit_size = LittleEndian::read_u32(&size_buff);
        let cloned_stream = self.tcp_stream.try_clone().unwrap();
        for byte in cloned_stream.bytes().take(self.transmit_size as usize) {
                self.input_buffer.push(byte.unwrap());
        }
    }
}

//Generate a new TcpSession to be returned to the calling executable
pub fn create(tcp_stream: net::TcpStream, buff_size: usize) -> TcpSession {
    let input_buffer: Vec<u8> = Vec::with_capacity(buff_size);
    let output_buffer: Vec<u8> = Vec::with_capacity(buff_size);
    let orders: Vec<u8> = vec![0;16];
    TcpSession::new(tcp_stream, 0, input_buffer, output_buffer, orders)
}
