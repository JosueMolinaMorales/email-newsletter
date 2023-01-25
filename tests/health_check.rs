use std::net::TcpListener;

use email_newsletter::run;

#[tokio::test]
async fn health_check_test() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let res = client
        .get(address)
        .send()
        .await
        .expect("Failted to execute request");

    assert!(res.status().is_success());
    assert!(res.text().await.unwrap() == "Welcome to the Email Newsletter API v0.0.1-alpha!".to_string());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind to address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}/", port)
}