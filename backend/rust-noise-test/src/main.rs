fn main() {
    static PATTERN: &'static str = "Noise_NN_25519_ChaChaPoly_BLAKE2b";

    let mut initiator = snow::Builder::new(PATTERN.parse().unwrap())
        .build_initiator()
        .expect("oui");
    let mut responder = snow::Builder::new(PATTERN.parse().unwrap())
        .build_responder()
        .expect("oui 2");

    let (mut read_buf, mut first_msg, mut second_msg) = ([0u8; 1024], [0u8; 1024], [0u8; 1024]);

    // -> e
    let len = initiator
        .write_message(&[], &mut first_msg)
        .expect("oui 3");

    // responder processes the first message...
    responder
        .read_message(&first_msg[..len], &mut read_buf)
        .expect("oui 4");

    // <- e, ee
    let len = responder
        .write_message(&[], &mut second_msg)
        .expect("oui 5");

    // initiator processes the response...
    initiator
        .read_message(&second_msg[..len], &mut read_buf)
        .expect("oui 6");

    // NN handshake complete, transition into transport mode.
    let mut initiator = initiator.into_transport_mode().unwrap();
    let mut responder = responder.into_transport_mode().unwrap();

    let mut buf_a = vec![0u8; 65535];
    let mut buf_b = vec![0u8; 65535];

    let len_a = initiator
        .write_message(b"Hello World !", &mut buf_a)
        .unwrap();

    let len_b = responder.read_message(&buf_a[0..len_a], &mut buf_b).expect("oops");

    println!("client said: {}", String::from_utf8_lossy(&buf_b[..len_b]));
}
