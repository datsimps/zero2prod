use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::{self, EmailClient};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Ensure that tracing is only initialized once using once cell
static TRACING: Lazy<()> = Lazy::new( || {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name, 
            default_filter_level,
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name, 
            default_filter_level,
            std::io::sink,
        );
        init_subscriber(subscriber);
    };
});

pub struct TestApp{
    pub address: String,
    //pub port: u16,
    pub db_pool: PgPool,
    //pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}


pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    //create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create databse");

    //migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");
    connection_pool
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    // Start a mock server to stand in for Postmark API
//    let email_server = MockServer::start().await;

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind address.");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration()
        .expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let server = zero2prod::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
/*

    // Randomize test config for isolation 
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        //c.email_client.base_url = email_server.uri();
        c
    };
    println!("uri: {}", &configuration.email_client.base_url);
    println!("con: {:?}", &configuration);
    // Create and migrate database
    configure_database(&configuration.database).await;
    
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let application_port = application.port();

    // Launch application as a background task
    let _ = tokio::spawn(application.run_until_stopped());
   
    TestApp { 
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
*/
}

