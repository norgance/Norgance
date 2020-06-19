fn main() {
    //static PATTERN: &'static str = "Noise_NN_25519_ChaChaPoly_BLAKE2b";
    //static PATTERN: &'static str = "Noise_NX_25519_XChaChaPoly_BLAKE2b";
    //static PATTERN: &'static str = "Noise_IX_25519_XChaChaPoly_BLAKE2b";
    //static PATTERN: &'static str = "Noise_XX_25519_XChaChaPoly_BLAKE2b";
    //static PATTERN: &'static str = "Noise_KX_25519_XChaChaPoly_BLAKE2b";
    static PATTERN: &'static str = "Noise_XK_25519_XChaChaPoly_BLAKE2b";
    //static SECRET_INITIATOR: &'static [u8] = b"lapins lapins lapins lapins lapi";
    //static SECRET_RESPONDER: &'static [u8] = b"canards canards canards canards.";
    //static SECRET_INITIATOR: &'static [u8] = b"i don't care for fidget spinners";


    let builder_initiator: snow::Builder<'_> = snow::Builder::new(PATTERN.parse().unwrap());
    //let builder_initiator_2: snow::Builder<'_> = snow::Builder::new(PATTERN.parse().unwrap());
    let builder_responder: snow::Builder<'_> = snow::Builder::new(PATTERN.parse().unwrap());

    /*let mut initiator = snow::Builder::new(PATTERN.parse().unwrap())
        .build_initiator()
        .expect("oui");*/

    let static_key_initiator = builder_initiator.generate_keypair().unwrap();//.private;
    //println!("{:?}", static_key_initiator.public);
    //println!("{:?}", static_key_initiator.private);
    //let static_key_initiator_2 = builder_initiator_2.generate_keypair().unwrap();
    //let static_key_responder = builder_responder.generate_keypair().unwrap();//.private;
    let responder_public_key : [u8; 32] = [146, 80, 229, 52, 194, 85, 84, 202, 11, 101, 171, 244, 151, 96, 183, 147, 232, 176, 19, 107, 196, 213, 36, 217, 67, 90, 64, 70, 38, 250, 227, 121];
    let responder_private_key : [u8; 32] = [13, 100, 222, 24, 193, 56, 205, 224, 221, 105, 5, 64, 190, 149, 63, 84, 201, 73, 235, 111, 22, 135, 214, 95, 3, 54, 46, 91, 194, 129, 178, 106];
    //println!("{:?}", static_key_initiator.public);
    //println!("{:?}", static_key_initiator.private);
    let mut initiator = builder_initiator
        .local_private_key(&static_key_initiator.private)
        //.psk(3, SECRET_INITIATOR)
        .remote_public_key(&responder_public_key)
        //.psk(3, SECRET_RESPONDER)
        .build_initiator()
        .expect("oui 1");


    let mut responder = builder_responder
        //.remote_public_key(&static_key_initiator.public)
        //.psk(3, SECRET_INITIATOR)
        .local_private_key(&responder_private_key)
        //.psk(3, SECRET_RESPONDER)
        .build_responder()
        .expect("oui 2");

    /*let mut responder = snow::Builder::new(PATTERN.parse().unwrap())
    .build_responder()
    .expect("oui 2");*/
    let (mut read_buf, mut first_msg, mut second_msg, mut third_msg) = ([0u8; 1024], [0u8; 1024], [0u8; 1024], [0u8; 1024]);

    // -> e
    let len = initiator.write_message(&[], &mut first_msg).expect("oui 3a");

    // responder processes the first message...
    responder
        .read_message(&first_msg[..len], &mut read_buf)
        .expect("oui 4");

    // <- e, ee, s, es
    let len = responder
        .write_message(&[], &mut second_msg)
        .expect("oui 5");
    
    //let len = initiator.write_message(&[], &mut second_msg).expect("oui 3b");

    // initiator processes the response...
    initiator
        .read_message(&second_msg[..len], &mut read_buf)
        .expect("oui 6");
   
    // -> s, se
    let len = initiator
        .write_message(&[], &mut third_msg)
        .expect("oui 7");
    
    let len = responder
        .read_message(&third_msg[..len], &mut read_buf)
        .expect("oui 8");

    // NN handshake complete, transition into transport mode.
    let mut initiator = initiator.into_transport_mode().unwrap();
    let mut responder = responder.into_transport_mode().unwrap();

    let mut buf_a = vec![0u8; 65535];
    let mut buf_b = vec![0u8; 65535];

    let mut len_a = initiator
        .write_message(b"Hello World !", &mut buf_a)
        .unwrap();

    let mut len_b = responder
        .read_message(&buf_a[0..len_a], &mut buf_b)
        .expect("oops");

    println!("client said: {}", String::from_utf8_lossy(&buf_b[..len_b]));

    len_a = responder.write_message(b"ok cool", &mut buf_a).unwrap();

    len_b = initiator
        .read_message(&buf_a[0..len_a], &mut buf_b)
        .expect("read oops");

    println!("server said: {}", String::from_utf8_lossy(&buf_b[..len_b]));
}
