use actix_web::{HttpResponse, http::header::ContentType, HttpRequest};
use actix_web::cookie::{Cookie, time::Duration};

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_hmtl = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => {
                format!("<p><i>{}</i></p>", cookie.value())
            }
    };
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!( 
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Login</title>
            </head>
            <body>
                {error_hmtl}
                <form action="/login" method="post">
                    <label>Username
                        <input type="text" placeholder="Enter Username" name="username">
                    </label>
                    <label>Password
                        <input type="text" placeholder="Enter Password" name="password">
                    </label>

                    <button type="submit">Login</button>
                </form>
            </body>
            </html>
            "#,
        ));
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
