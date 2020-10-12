fn main() {
    use std::net::TcpListener;
    use std::thread::spawn;
    use tungstenite::server::accept;
    static PATTERN: &'static str = "Noise_XK_25519_XChaChaPoly_BLAKE2b";

    static responder_private_key: [u8; 32] = [
        13, 100, 222, 24, 193, 56, 205, 224, 221, 105, 5, 64, 190, 149, 63, 84, 201, 73, 235, 111,
        22, 135, 214, 95, 3, 54, 46, 91, 194, 129, 178, 106,
    ];

    /// A WebSocket echo server
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let builder_responder: snow::Builder<'_> = snow::Builder::new(PATTERN.parse().unwrap());
            let mut responder = builder_responder
                //.remote_public_key(&static_key_initiator.public)
                //.psk(3, SECRET_INITIATOR)
                .local_private_key(&responder_private_key)
                //.psk(3, SECRET_RESPONDER)
                .build_responder()
                .expect("oui 2");

            //let (mut read_buf, mut first_msg, mut second_msg, mut third_msg) = ([0u8; 1024], [0u8; 1024], [0u8; 1024], [0u8; 1024]);

            let a = websocket.read_message().unwrap();
            if !a.is_binary() {
                panic!("naaah")
            }

            let mut buffer_a = [0u8; 1024];
            responder
                .read_message(&a.into_data(), &mut buffer_a)
                .expect("oui 3");

            let mut buffer_b = [0u8; 1024];
            let len = responder.write_message(&[], &mut buffer_b).expect("oui 4");

            websocket
                .write_message(tungstenite::Message::Binary(buffer_b[..len].to_vec()))
                .expect("ouiii 4");

            let b = websocket.read_message().unwrap();
            if !b.is_binary() {
                panic!("naaaaah 2");
            }

            let mut buffer_c = [0u8; 1024];
            responder.read_message(&b.into_data(), &mut buffer_c)
                .expect("ouiii 5");

            let mut responder = responder.into_transport_mode().expect("ouiii 6");

            loop {
                let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}
