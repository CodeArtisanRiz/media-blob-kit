use axum::response::Html;
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct RootResponse {
    pub message: String,
    pub version: String,
    pub endpoints: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Welcome page HTML", content_type = "text/html")
    ),
    tag = "General"
)]
pub async fn root() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>MediaBlobKit</title>
            <style>
                body {
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    font-family: Arial, sans-serif;
                    background-color: #f0f0f0;
                }
                h1 {
                    color: #333;
                }
                p {
                    color: #666;
                }
            </style>
        </head>
        <body>
            <h1>Welcome to MediaBlobKit</h1>
            <p>Your solution for media blob management.</p>
            <a href="/swagger-ui/" style="
                margin-top: 20px;
                padding: 10px 20px;
                background-color: #007bff;
                color: white;
                text-decoration: none;
                border-radius: 5px;
                font-weight: bold;
                transition: background-color 0.3s;
            " onmouseover="this.style.backgroundColor='#0056b3'" onmouseout="this.style.backgroundColor='#007bff'">
                Explore API Docs
            </a>
        </body>
        </html>
    "#)
}
