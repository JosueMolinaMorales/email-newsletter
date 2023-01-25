use std::net::TcpListener;

use email_newsletter::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    run(TcpListener::bind("127.0.0.1:8080")?)?.await
}