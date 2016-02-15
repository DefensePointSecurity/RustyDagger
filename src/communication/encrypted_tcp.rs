use std::io::{Write, Read};
use byteorder::{ByteOrder, LittleEndian};
use data_mod::*;
use communication::*;

pub struct EncryptedSession {
    pub tcp_session: tcp_connection::TcpSession,
    pub keyring: encryption::KeyRing,
    pub size_buffer: Vec<u8>,
}

impl EncryptedSession {
    
    fn new(tcp_session: tcp_connection::TcpSession, keyring: encryption::KeyRing, size_buffer: Vec<u8>) -> EncryptedSession {
        EncryptedSession {
			tcp_session: tcp_session,
            keyring: keyring,
			size_buffer: size_buffer,
        }
    }
    
    //Attempt to write all buffered data to the TcpStream
    pub fn send(&mut self) -> () {
        encrypt(self);
        self.tcp_session.tcp_stream.write_all(&self.size_buffer).unwrap();
        self.tcp_session.tcp_stream.write(&self.tcp_session.output_buffer).unwrap();
        self.tcp_session.output_buffer.clear();
    }

    //Attempt to read data received from the TcpStream
    pub fn recv(&mut self) -> () {
        self.tcp_session.input_buffer.clear();
        self.size_buffer.clear();
        let cloned_stream_size = self.tcp_session.tcp_stream.try_clone().unwrap(); 
        for byte in cloned_stream_size.bytes().take(16) {
            self.size_buffer.push(byte.unwrap());
        }
        decrypt(&mut self.size_buffer, &self.keyring);
        self.tcp_session.transmit_size = LittleEndian::read_u32(&self.size_buffer[0..4]);
        let cloned_stream_data = self.tcp_session.tcp_stream.try_clone().unwrap();
        for byte in cloned_stream_data.bytes().take(self.tcp_session.transmit_size as usize) {
                self.tcp_session.input_buffer.push(byte.unwrap());
        }
        decrypt(&mut self.tcp_session.input_buffer, &self.keyring);
    }

    pub fn get_orders(&mut self) -> u32 {
        self.tcp_session.input_buffer.clear();
        self.tcp_session.orders.clear();
        let cloned_stream = self.tcp_session.tcp_stream.try_clone().unwrap(); 
        for byte in cloned_stream.bytes().take(16) {
            self.tcp_session.orders.push(byte.unwrap());
        }
        decrypt(&mut self.tcp_session.orders, &self.keyring);
        LittleEndian::read_u32(&self.tcp_session.orders[0..4])
    }

    pub fn send_orders(&mut self) -> () {
        for byte in self.tcp_session.orders[0..4].iter() {
            self.tcp_session.output_buffer.push(*byte);
        }
        encrypt(self);
        self.tcp_session.tcp_stream.write(&self.tcp_session.output_buffer).unwrap();
        self.tcp_session.output_buffer.clear();
    }
}

//Generate a new EncryptedSession to be returned to the calling executable
pub fn create(tcp_session: tcp_connection::TcpSession, keyring: encryption::KeyRing) -> EncryptedSession {
    let size_buffer: Vec<u8> = vec![0;16];
    EncryptedSession::new(tcp_session, keyring, size_buffer)
}

fn encrypt(encrypted_session: &mut encrypted_tcp::EncryptedSession) -> () {
    let mut encrypted_data = encryption::encrypt(&encrypted_session.tcp_session.output_buffer, &encrypted_session.keyring)
        .ok()
        .unwrap();
    LittleEndian::write_u32(&mut encrypted_session.size_buffer, encrypted_data.len() as u32);
    let mut encrypted_len = encryption::encrypt(&encrypted_session.size_buffer[0..4], &encrypted_session.keyring)
        .ok()
        .unwrap();
    encrypted_session.tcp_session.output_buffer.clear();
    encrypted_session.size_buffer.clear();
    encrypted_session.size_buffer.shrink_to_fit();
    encrypted_session.tcp_session.output_buffer.append(&mut encrypted_data);
    encrypted_session.size_buffer.append(&mut encrypted_len);
}

fn decrypt(data_buffer: &mut Vec<u8>, keyring: &encryption::KeyRing) -> () {
    let mut decrypted_data = encryption::decrypt(&data_buffer, &keyring)
        .ok()
        .unwrap();
    data_buffer.clear();
    data_buffer.append(&mut decrypted_data);
}
