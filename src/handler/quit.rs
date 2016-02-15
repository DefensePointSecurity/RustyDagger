use communication::*;
use byteorder::{ByteOrder, LittleEndian};

pub fn client_run(encrypted_session: &mut encrypted_tcp::EncryptedSession) -> () {
    LittleEndian::write_u32(&mut encrypted_session.tcp_session.orders, 1001 as u32);
    encrypted_session.send_orders();
}
