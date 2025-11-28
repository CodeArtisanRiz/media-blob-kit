mod routes;

use routes::create_routes;

#[tokio::main]
async fn main() {
    // build our application using the routes module
    let app = create_routes();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
