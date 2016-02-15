use std::process::Command;
use std::str;
use communication::*;
use byteorder::{ByteOrder, LittleEndian};

pub fn client_run(encrypted_session: &mut encrypted_tcp::EncryptedSession, command_string: &str) -> () {
    if command_string.is_empty() == true {
        return
    }
    LittleEndian::write_u32(&mut encrypted_session.tcp_session.orders, 1000 as u32);
    encrypted_session.send_orders();
    for byte in command_string.as_bytes().iter() {
        encrypted_session.tcp_session.output_buffer.push(*byte);
    }
    encrypted_session.send();//Send command string
    encrypted_session.recv();//Receive output of command
}

pub fn server_run(encrypted_session: &mut encrypted_tcp::EncryptedSession) -> Result<(), str::Utf8Error> {
    encrypted_session.recv();
    let mut command_vector: Vec<&str> = try!(str::from_utf8(&encrypted_session.tcp_session.input_buffer))
        .split_whitespace()
        .collect();
    if command_vector.is_empty() {
        return Ok(())
    } //Make sure the client didn't send us a blank line.
    let command_name = command_vector[0];
    command_vector.remove(0);
    if command_vector.is_empty() == true {
        match Command::new(command_name).output() {
            Ok(mut out) => { 
                encrypted_session.tcp_session.output_buffer.append(&mut out.stdout);
                encrypted_session.tcp_session.output_buffer.append(&mut out.stderr);
            },
            Err(err) => {
                encrypted_session.tcp_session.output_buffer.append(&mut err.to_string().into_bytes());
                encrypted_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
            },
        };
    } else {
        match Command::new(command_name).args(&command_vector).output() {
            Ok(mut out) => {
                encrypted_session.tcp_session.output_buffer.append(&mut out.stdout);
                encrypted_session.tcp_session.output_buffer.append(&mut out.stderr);
            },
            Err(err) => {
                encrypted_session.tcp_session.output_buffer.append(&mut err.to_string().into_bytes());
                encrypted_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
            },
        };
    };
    Ok(())
}
