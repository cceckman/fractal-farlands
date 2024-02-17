use ff_web::root_routes;

fn main() {
    let web_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("could not construct Tokio runtime");
    let handle = web_rt.handle().clone();

    let server = async {
        let app = root_routes(handle);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        axum::serve(listener, app).await
    };
    web_rt.block_on(server).expect("server terminated");
}
