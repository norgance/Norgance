use actix_http::ResponseBuilder;
use actix_web::{
    error, http::StatusCode, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use env_logger::Env;
use failure::Fail;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::sync::Arc;

struct AppState {
    lines_count: usize,
    hashes_file_path: Arc<String>,
    index: Arc<Vec<usize>>,
}

#[derive(Fail, Debug)]
#[fail(display = "password quality error")]
enum PasswordQualityError {
    #[fail(display = "internal server error")]
    InternalError,
    #[fail(display = "bad hash prefix")]
    BadHashPrefix,
    #[fail(display = "unsupported format")]
    InsupportedFormat,
}

#[derive(Serialize)]
struct PasswordQualityErrorResponse {
    error: String,
}

impl error::ResponseError for PasswordQualityError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(PasswordQualityErrorResponse {
            error: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            PasswordQualityError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            PasswordQualityError::BadHashPrefix => StatusCode::BAD_REQUEST,
            PasswordQualityError::InsupportedFormat => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
}

async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: String::from("ok"),
    })
}

#[derive(Serialize)]
struct HashesSuffixes {
    suffixes: Vec<HashSuffix>,
}

#[derive(Serialize)]
struct HashSuffix {
    suffix: String,
    quality: String,
}

async fn password_quality(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let prefix = req.match_info().get("prefix").expect("prefix");
    if prefix.len() != 5 {
        return Err(PasswordQualityError::BadHashPrefix);
    }

    let index = &data.index;
    let hashes_file_path = (&data.hashes_file_path).to_string();
    let lines_count = data.lines_count;

    let (from, to) = find_in_index(String::from(prefix), &index, lines_count)?;
    let (from_padded, to_padded) = add_padding(from, to, 1000, lines_count);
    let hashes = load_hashes(hashes_file_path, from_padded, to_padded)?;

    let format = req.match_info().get("format").unwrap_or("text");
    match format {
        "json" => {
            let json = convert_plain_hashes_to_json(hashes);
            return Ok::<HttpResponse, PasswordQualityError>(HttpResponse::Ok().json(json));
        }
        "csv" => {
            return Ok::<HttpResponse, PasswordQualityError>(
                HttpResponse::Ok()
                    .content_type("application/csv")
                    .body(hashes),
            );
        }
        "text" | "txt" | "plain" => {
            return Ok::<HttpResponse, PasswordQualityError>(
                HttpResponse::Ok().content_type("text/plain").body(hashes),
            );
        }
        _ => {
            return Err(PasswordQualityError::InsupportedFormat);
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

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

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(AppState {
                lines_count: lines_count,
                index: Arc::clone(&index_arc),
                hashes_file_path: Arc::clone(&hashes_file_path_arc),
            })
            .route("/", web::get().to(healthcheck))
            .route(
                "/password_quality/{prefix}.{format}",
                web::get().to(password_quality),
            )
            .route(
                "/password_quality/{prefix}",
                web::get().to(password_quality),
            )
    })
    .bind(format!("0.0.0.0:{}", http_port))?
    .run()
    .await
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

fn find_in_index(
    hash_prefix: String,
    index: &Vec<usize>,
    lines_count: usize,
) -> Result<(usize, usize), PasswordQualityError> {
    let hash_index = match usize::from_str_radix(&hash_prefix, 16) {
        Ok(x) => x,
        Err(_) => return Err(PasswordQualityError::BadHashPrefix),
    };

    if hash_index >= index.len() {
        return Err(PasswordQualityError::BadHashPrefix);
    }

    let from = index[hash_index];

    if hash_index == index.len() - 1 {
        return Ok((from, lines_count));
    }

    let to = index[hash_index + 1] - 1;
    return Ok((from, to));
}

fn load_hashes(hash_path: String, from: usize, to: usize) -> Result<Vec<u8>, PasswordQualityError> {
    let mut hashes_file = match File::open(hash_path) {
        Ok(x) => x,
        Err(_) => return Err(PasswordQualityError::InternalError),
    };

    let count = to - from;
    const BYTES_PER_LINE: usize = 38; // 43

    let mut buffer = vec![0; count * BYTES_PER_LINE];

    match hashes_file.seek(SeekFrom::Start((from * BYTES_PER_LINE) as u64)) {
        Err(_) => return Err(PasswordQualityError::InternalError),
        _ => {}
    }
    match hashes_file.read(&mut buffer) {
        Err(_) => return Err(PasswordQualityError::InternalError),
        _ => {}
    }

    return Ok(buffer);
}

fn convert_plain_hashes_to_json(hashes: Vec<u8>) -> HashesSuffixes {
    const BYTES_PER_LINE: usize = 38;
    const SUFFIX_LENGTH: usize = 35;
    const QUALITY_POSITION: usize = 36;
    let hashes_count = hashes.len() / BYTES_PER_LINE;

    let mut suffixes = Vec::with_capacity(hashes_count);
    for i in 0..hashes_count {
        suffixes.push(HashSuffix {
            suffix: String::from_utf8_lossy(
                &hashes[i * BYTES_PER_LINE..i * BYTES_PER_LINE + SUFFIX_LENGTH],
            )
            .into_owned(),
            quality: String::from_utf8_lossy(
                &hashes[i * BYTES_PER_LINE + QUALITY_POSITION
                    ..i * BYTES_PER_LINE + QUALITY_POSITION + 1],
            )
            .into_owned(),
        })
    }

    HashesSuffixes { suffixes: suffixes }
}

fn add_padding(from: usize, to: usize, target: usize, lines_count: usize) -> (usize, usize) {
    let size = to - from;
    if size >= target {
        return (from, to);
    }
    let missing = target - size;

    if to + missing > lines_count {
        return (from - missing, to);
    }
    return (from, to + missing);
}
