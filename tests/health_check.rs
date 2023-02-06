
#[tokio::test]
async fn health_check_test() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/", test_app.address))
        .send()
        .await
        .expect("Failted to execute request");

    assert!(res.status().is_success());
    assert!(
        res.text().await.unwrap()
            == "Welcome to the Email Newsletter API v0.0.1-alpha!".to_string()
    );
}



