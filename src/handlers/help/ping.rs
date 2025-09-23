#[utoipa::path(
    get,
    path = "/api/help/ping",
    responses(
        (status = 200, description = "Ping successful", body = String)
    ),
    tag = "Help"
)]
pub async fn ping() -> &'static str {
    "pong"
}
