use crypto::{ symmetriccipher, buffer, aes, blockmodes, ed25519 };
use crypto::buffer:: { ReadBuffer, WriteBuffer, BufferResult };
use rand::{ Rng, OsRng };
use communication::*;
use data_mod::*;

pub struct KeyRing {
    private_key: [u8; 64],
    public_key: [u8; 32],
    shared_key: [u8; 32],
    iv: [u8; 16],
}

impl KeyRing {

    fn new(private_key: [u8; 64], public_key: [u8; 32], shared_key: [u8; 32], iv: [u8; 16]) -> KeyRing {
        KeyRing {
            private_key: private_key,
            public_key: public_key,
            shared_key: shared_key,
            iv: iv,
        }
    }
}

pub fn create_keyring() -> KeyRing {
    let mut csrng = OsRng::new().unwrap();
    let (private_key, public_key) = gen_keypair(&mut csrng);
    let iv: [u8;16] = [0;16];
    let shared_key: [u8;32] = [0;32];
    KeyRing::new(private_key, public_key, shared_key, iv)
}

fn gen_keypair(csrng: &mut OsRng) -> ([u8; 64], [u8; 32]) {
    let mut seed: [u8;32] = [0;32];
    csrng.fill_bytes(&mut seed);
    ed25519::keypair(&seed)
}

pub fn send_pubkey(tcp_session: &mut tcp_connection::TcpSession, keyring: &encryption::KeyRing) -> () {
    for byte in &keyring.public_key {
        tcp_session.output_buffer.push(*byte);
    }
    tcp_session.send()
}

pub fn decrypt(encrypted_data: &[u8], keyring: &encryption::KeyRing) -> 
Result<Vec<u8>, symmetriccipher::SymmetricCipherError>{
    
    let mut decryptor = aes::cbc_decryptor(aes::KeySize::KeySize256, &keyring.shared_key, &keyring.iv, blockmodes::PkcsPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().cloned());
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}

pub fn encrypt(data: &[u8], keyring: &encryption::KeyRing) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {

    let mut encryptor = aes::cbc_encryptor(aes::KeySize::KeySize256, &keyring.shared_key, &keyring.iv, blockmodes::PkcsPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().cloned());
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }
    
    Ok(final_result)
}

pub fn gen_sharedkey(their_public: &[u8], keyring: &mut encryption::KeyRing) -> () {
    keyring.shared_key = ed25519::exchange(&their_public, &keyring.private_key);
}

pub fn send_iv(tcp_session: &mut tcp_connection::TcpSession, keyring: &mut encryption::KeyRing) -> () {
    let mut csrng = OsRng::new().unwrap();
    csrng.fill_bytes(&mut keyring.iv);
    for byte in &keyring.iv {
        tcp_session.output_buffer.push(*byte);
    }
    tcp_session.send()
}

pub fn get_iv(tcp_session: &mut tcp_connection::TcpSession, keyring: &mut encryption::KeyRing) -> () {
    tcp_session.recv();
    let mut count = 0;
    while count < keyring.iv.len() {
        keyring.iv[count] = tcp_session.input_buffer[count];
        count = count + 1;
    }
}
