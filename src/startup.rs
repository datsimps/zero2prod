use crate::routes::{health_check, subscribe};
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
    ) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
