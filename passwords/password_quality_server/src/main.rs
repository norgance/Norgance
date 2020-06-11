use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "password_quality=info");
    }
    pretty_env_logger::init();

    let index_file_path: String = env::var_os("PASSWORD_INDEX_PATH")
        .unwrap_or(std::ffi::OsString::from("index.csv"))
        .into_string()
        .expect("Unable to load index path from environment");
    let hashes_file_path: String = env::var_os("PASSWORD_HASHES_PATH")
        .unwrap_or(std::ffi::OsString::from("hashes.csv"))
        .into_string()
        .expect("Unable to load password path from environment");

    let http_port: u16 = env::var_os("HTTP_PORT")
        .unwrap_or(std::ffi::OsString::from("3030"))
        .into_string()
        .expect("Unable to load http port from environment")
        .parse::<u16>()
        .expect("Unable to parse http port from environment");

    let (index, lines_count) = build_index(index_file_path);

    let index_arc = Arc::new(index);
    let hashes_file_path_arc = Arc::new(hashes_file_path);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("password_quality" / String)
        .map(move |name: String| {
            let hash_index = usize::from_str_radix(&name, 16).expect("Unable to parse the hash");
            let local_index = Arc::clone(&index_arc);
            let local_hashes_file_path = Arc::clone(&hashes_file_path_arc).to_string();

            let (from, to) = find_in_index(name, &local_index);

            //let hashes = load_hashes(local_hashes_file_path, from, to.min(lines_count));
            let hashes = load_hashes(local_hashes_file_path, from, to.min(from + 10));

            //let truncated_hashes = truncate_hashes(hashes);
            //let json_hashes = truncates_hashes_and_make_json(hashes);
            // let data = std::str::from_utf8(&buffer).expect("utf8 conversion failed");

            return warp::reply::with_header(hashes, "content-type", "text/plain");
            // return warp::reply::with_header(json_hashes, "content-type", "application/json");
        })
        .with(warp::log("password_quality"))
        .recover(handle_rejection);

    println!("🌹 Server starting on port {}", http_port);
    warp::serve(hello).run(([0, 0, 0, 0], http_port)).await;

    // println!("{}", index.len());
}

fn build_index(index_file_path: String) -> (Vec<usize>, usize) {
    let index_file = File::open(index_file_path).expect("Unable to open index file");
    let input_buffer = BufReader::new(index_file);

    const INDEX_SIZE: usize = 1048576;
    let mut array: Vec<usize> = vec![0; INDEX_SIZE];

    let mut current_hash_index: usize = 0;
    let mut current_hash_line: usize = 0;

    for line in input_buffer.lines() {
        let unwrapped_line = line.expect("Unreadable line");
        let mut fields = unwrapped_line.split(',');
        let hash_prefix = fields.next().expect("We are missing a hash on a line");
        let hash_index = usize::from_str_radix(hash_prefix, 16).expect("Unable to parse the hash");

        if hash_index > INDEX_SIZE {
            panic!("Parsed hash is too high")
        }

        if hash_index != 0 && hash_index != current_hash_index + 1 {
            panic!("Hash indexes are not following !")
        }

        let start = fields.next().expect("start line is missing");
        let start_int = start
            .trim()
            .parse::<usize>()
            .expect("Unable to parse start line");
        let to = fields.next().expect("to line is missing");
        let to_int = to.trim().parse::<usize>().expect("Unable to parse to line");

        if start_int != 0 && start_int != current_hash_line + 1 {
            panic!("Lines are not following in the dataset !")
        }

        current_hash_index = hash_index;
        array[hash_index] = start_int;
        current_hash_line = to_int;
    }

    return (array, current_hash_line);
}

fn find_in_index(hash_prefix: String, index: &Vec<usize>) -> (usize, usize) {
    let hash_index = usize::from_str_radix(&hash_prefix, 16).expect("Unable to parse the hash");

    if hash_index > index.len() {
        panic!("index too high");
    }

    if hash_index == index.len() - 1 {
        panic!("not supported")
    }

    let from = index[hash_index];
    let to = index[hash_index + 1] - 1;

    return (from, to);
}

fn load_hashes(hash_path: String, from: usize, to: usize) -> Vec<u8> {
    let mut hashes_file = File::open(hash_path).expect("Unable to open hashes file");
    let count = to - from;
    const BYTES_PER_LINE: usize = 38; // 43

    let mut buffer = vec![0; count * BYTES_PER_LINE];

    hashes_file
        .seek(SeekFrom::Start((from * BYTES_PER_LINE) as u64))
        .expect("prout seek");
    //hashes_file.seek(SeekFrom::Start(43)).expect("prout seek");
    hashes_file.read(&mut buffer).expect("prout read");

    return buffer;
}

/*fn truncate_hashes(input: Vec<u8>) -> Vec<u8> {
    const BYTES_PER_LINE: usize = 43;
    const BYTES_PER_TRUNCATED_LINE: usize = BYTES_PER_LINE - 5;

    let lines_count = input.len() / BYTES_PER_LINE;

    let mut buffer = vec![0; lines_count * BYTES_PER_TRUNCATED_LINE];

    // Copy the buffers accordingly
    for i in 0..lines_count {
        buffer[i * BYTES_PER_TRUNCATED_LINE..(i + 1) * BYTES_PER_TRUNCATED_LINE]
            .copy_from_slice(&input[i * BYTES_PER_LINE + 5..(i + 1) * BYTES_PER_LINE]);
    }

    return buffer;
}*/

/*fn truncates_hashes_and_make_json(input: Vec<u8>) -> Vec<u8> {
    const TRUNCATION_LINE_START : usize = 5;
    const TRUNCATION_LINE_END : usize = 3;
    const BYTES_PER_LINE: usize = 43;
    const BYTES_PER_TRUNCATED_LINE: usize = BYTES_PER_LINE - TRUNCATION_LINE_START - TRUNCATION_LINE_END;
    // {"suffixes":[]}
    const JSON_CONTAINER_OVERHEAD: usize = 15;
    // {"suffix":"","score":""},{ ,
    const JSON_LINE_OVERHEAD: usize = 25;
    const BYTES_PER_JSON_LINE: usize = BYTES_PER_TRUNCATED_LINE + JSON_LINE_OVERHEAD;

    let lines_count = input.len() / BYTES_PER_LINE;
    // We remove 1 because we need to remove the last comma
    let buffer_size = lines_count * BYTES_PER_JSON_LINE + JSON_CONTAINER_OVERHEAD - 1;
    let mut buffer = vec![0; buffer_size];


    const JSON_CONTAINER_PREFIX: &[u8] = "{\"suffixes\":[".as_bytes();
    const JSON_CONTAINER_PREFIX_LENGTH: usize = JSON_CONTAINER_PREFIX.len();
    const JSON_CONTAINER_SUFFIX: &[u8] = "]}".as_bytes();
    const JSON_CONTAINER_SUFFIX_LENGTH: usize = JSON_CONTAINER_SUFFIX.len();

    buffer[0..JSON_CONTAINER_PREFIX_LENGTH].copy_from_slice(JSON_CONTAINER_PREFIX);

    const JSON_LINE_PREFIX: &[u8] = "{\"suffix\":\"".as_bytes();
    const JSON_LINE_PREFIX_LENGTH: usize = JSON_LINE_PREFIX.len();
    const JSON_LINE_SUFFIX: &[u8] = "\",\"score\":\"?\"},".as_bytes();
    const JSON_LINE_SUFFIX_LENGTH: usize = JSON_LINE_SUFFIX.len();

    const MIDDLE_INDEX: usize = JSON_LINE_PREFIX_LENGTH + BYTES_PER_TRUNCATED_LINE;// - TRUNCATION_LINE_END;

    // Copy the buffers accordingly
    for i in 0..lines_count {
        let start = JSON_CONTAINER_PREFIX_LENGTH + i * BYTES_PER_JSON_LINE;
        let middle = start + MIDDLE_INDEX;
        buffer[start..start + JSON_LINE_PREFIX_LENGTH].copy_from_slice(JSON_LINE_PREFIX);
        buffer[start + JSON_LINE_PREFIX_LENGTH..middle]
            .copy_from_slice(&input[i * BYTES_PER_LINE + TRUNCATION_LINE_START..(i + 1) * (BYTES_PER_LINE) - TRUNCATION_LINE_END]);
        buffer[middle..middle + JSON_LINE_SUFFIX_LENGTH].copy_from_slice(JSON_LINE_SUFFIX);
        buffer[middle+11] = input[i * BYTES_PER_LINE + 41];
    }

    // We overide the end of the buffer with the container suffix
    buffer[buffer_size - JSON_CONTAINER_SUFFIX_LENGTH..buffer_size].copy_from_slice(JSON_CONTAINER_SUFFIX);
    
    return buffer;
}*/
use warp::{reject, Rejection, Reply};
use warp::http::StatusCode;
use std::convert::Infallible;

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    /*} else if let Some(DivideByZero) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "DIVIDE_BY_ZERO";*/
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    Ok(warp::reply::with_status(format!("{}: {}", code, message), code))
}
