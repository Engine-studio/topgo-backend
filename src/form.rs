use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer, HttpRequest, Responder,
};

use actix_web_dev::error::{
    Result,
    ApiError,
    ErrorType,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LandingRequest {
    name: String,
    phone: String,
    mail: String,
    text: String,
}

pub async fn create(
    form: web::Json<LandingRequest>,
) -> Result<()> {
    let form = form.into_inner();
    send_auth_link(&form).await;
    Ok(())
}

pub async fn send_auth_link(form: &LandingRequest) -> Result<()>  {
    
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};
    use lettre::message::{header, MultiPart, SinglePart};

    let html = format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello from Lettre!</title>
</head>
<body>
    <p>Имя: {}<p><br/>
    <p>Телефон: {}<p><br/>
    <p>Мейл: {}<p><br/>
    <p>Текст: {}<p><br/>
</body>
</html>"#,form.name,form.phone,form.mail,form.text);

    println!("msg {}",html);
    let email = Message::builder()
    .from("topgo-noreply@yandex.ru".parse().unwrap())
    .to("topgo-noreply@yandex.ru".parse().unwrap())
    .subject("New Request")
    .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::parse("text/plain; charset=utf8")
                            .unwrap())
                        .body(format!("Name: {}\nMail: {}\nPhone: {}\nText: {}",
                                form.name,form.mail,form.phone,form.text).to_string()), 
                        // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::parse(
                            "text/html; charset=utf8").unwrap())
                        .body(html.to_string()),
                ),
    ).map_err(|e| {
        ApiError {
            code: 500,
            message: "err building msg".to_string(),
            error_type: ErrorType::InternalError,
        }
    })?;

    let creds = Credentials::new("topgo-noreply@yandex.ru".to_string(), 
        "xarres-moMtuv-qodme6".to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp-pulse.com")
        .unwrap()
        .credentials(creds)
        .build();   

    // Send the email
    let _ = mailer.send(&email).map_err(|e|{
        ApiError {
            code: 500,
            message: "err building msg".to_string(),
            error_type: ErrorType::InternalError,
        }
    })?;
    Ok(())
}

