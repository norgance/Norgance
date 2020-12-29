fn main() -> anyhow::Result<()> {
    /*use std::convert::TryFrom;
    use ed25519_dalek::Verifier;
    use anyhow::Context;

    let public_key_base64 = "vbwUgKPXHXB00lAv7xd6QJNbWNvLquyAa0qCkTtvscY=";
    let public_key_bytes = base64::decode(public_key_base64)?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)?;

    let message_base64 = "SGVsbG8gV29ybGQ=";
    let message = base64::decode(message_base64)?;

    let signature = "yx84ycvzCWckNpjf54s3QJGFAH4nGOatyZhMR/nQXJmqfwLm7BHYGbI2+TzImKmKU2eYiZc2zUP0frqZLaBdBA==";
    let signature_bytes = base64::decode(signature).context("prout")?;
    let signature = ed25519_dalek::Signature::try_from(&signature_bytes[..])?;

    match public_key.verify(&message, &signature) {
        Ok(_) => println!("OK"),
        Err(x) => println!("ERROR: {}",x),
    }*/

    use ed25519_dalek::Verifier;
    use std::convert::TryFrom;
    use std::io::prelude::*;

    let mut port = serialport::new("/dev/pts/3", 9600)
        .timeout(std::time::Duration::from_millis(30000))
        .open()?;

    /*let mut port = std::net::TcpStream::connect("192.168.10.169:5421")?;

    port.read(&mut [0; 128])?;*/

    let mut reader = std::io::BufReader::new(port.try_clone()?);
    let mut line = String::new();

    // Health check
    port.write(b"CANARD\n")?;
    reader.read_line(&mut line)?;
    assert_eq!(line.trim(), "KOINKOIN");
    println!("Connected");

    port.write(b"GET_PUBLIC_KEY\n")?;
    line.clear();
    reader.read_line(&mut line)?;
    println!("Public Key: {}", line.trim());
    let public_key_bytes = base64::decode(line.trim())?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)?;

    /*port.write(b"RENEW_KEYS\n")?;
    line.clear();
    reader.read_line(&mut line)?;
    let mut line_tokens = line.split_whitespace();
    assert_eq!(Some("PUBLIC_KEY"), line_tokens.next());
    let public_key_base64 = line_tokens.next().unwrap();
    let public_key_bytes = base64::decode(public_key_base64).unwrap();
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();*/

    for i in 0..4096 {
        // use rand::Rng;
        // let payload = rand::thread_rng().gen::<[u8; 32]>();
        let payload: Vec<u8> = (0..96).map(|_| { rand::random::<u8>() }).collect();
        let payload_base64 = base64::encode(&payload);
        println!("{}: {}", i, payload_base64);
        let command = format!("SIGN {}\n", payload_base64);
        port.write(command.as_bytes())?;
        line.clear();
        reader.read_line(&mut line)?;
        println!("RESULT: {}", line.trim());
        let signature_bytes = base64::decode(line.trim())?;
        let signature = ed25519_dalek::Signature::try_from(&signature_bytes[..])?;

        match public_key.verify(&payload, &signature) {
            Ok(_) => println!("{} OK", i),
            Err(x) => println!("{} ERROR: {}", i, x),
        }
    }

    Ok(())
}
