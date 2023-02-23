use actix_htmx::{HtmxDetails, HtmxMiddleware, TriggerType};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(HtmxMiddleware)
            .service(web::resource("/").to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn index(htmx_details: HtmxDetails) -> impl Responder {
    let is_htmx = htmx_details.is_htmx();
    let boosted = htmx_details.boosted();
    let current_url = htmx_details.current_url().unwrap_or_default();
    let history_restore_request = htmx_details.history_restore_request();
    let prompt = htmx_details.prompt().unwrap_or_default();
    let target = htmx_details.target().unwrap_or_default();
    let trigger = htmx_details.trigger().unwrap_or_default();
    let trigger_name = htmx_details.trigger_name().unwrap_or_default();

    htmx_details.trigger_event(
        "test".to_string(),
        "my event message!".to_string(),
        TriggerType::Standard,
    );
    htmx_details.trigger_event(
        "test2".to_string(),
        r#"{"level": "info", "message": "my event message!"}"#.to_string(),
        TriggerType::Standard,
    );
    htmx_details.trigger_event(
        "event3".to_string(),
        "my event message!".to_string(),
        TriggerType::AfterSettle,
    );
    htmx_details.trigger_event(
        "event4".to_string(),
        "another event message!".to_string(),
        TriggerType::AfterSwap,
    );
    htmx_details.redirect("/my_redirect".to_string());
    htmx_details.redirect_with_swap("/another_redirect".to_string());
    htmx_details.refresh();
    htmx_details.push_url("/push".to_string());
    HttpResponse::Ok().content_type("text/html").body(format!(
        r#"<!DOCTTYPE html>
<html>
    <head>
        <title>Actix HTMX Example</title>
        <script src="https://unpkg.com/htmx.org@latest"></script>
    </head>
    <body>
        <div>Is HTMX: {is_htmx}</div>
        <div>Boosted: {boosted}</div>
        <div>Current URL: {current_url}</div>
        <div>History Restore Request: {history_restore_request}</div>
        <div>Prompt: {prompt}</div>
        <div>Target: {target}</div>
        <div>Trigger: {trigger}</div>
        <div>Trigger Name: {trigger_name}</div>
    <body>
</html>
            "#
    ))
}
