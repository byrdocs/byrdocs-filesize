use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct FileInfo {
    filename: String,
    size: u64,
}

async fn get_filesize(data: web::Data<AppState>) -> impl Responder {
    let mut files_info = Vec::new();

    for entry in walkdir::WalkDir::new(&data.path) {
        let entry = entry.expect("Could not read entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name() {
                let filename = filename.to_string_lossy().to_string();
                if data.regex.is_match(&filename) {
                    let metadata = fs::metadata(&path).expect("Could not read metadata");
                    let size = metadata.len();
                    files_info.push(FileInfo {
                        filename,
                        size,
                    });
                }
            }
        }
    }

    HttpResponse::Ok().json(files_info)
}
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not Found")
}

struct AppState {
    path: PathBuf,
    regex: Regex,
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname
    #[arg(short='b', long, default_value_t=String::from("127.0.0.1"))]
    host: String,

    /// Port
    #[arg(short, long, default_value_t=8080)]
    port: u16,

    /// The path to scan for files
    path: String,

    /// The regex pattern to match file names
    regex: String,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let app_state = web::Data::new(AppState {
        path: PathBuf::from(args.path),
        regex: Regex::new(&args.regex).expect("Invalid regex"),
    });

    println!("Starting server at: {}:{}", args.host, args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/filesize.json", web::get().to(get_filesize))
            .default_service(web::route().to(not_found))
    })
    .bind(format!("{}:{}", args.host, args.port))?
    .run()
    .await
}
