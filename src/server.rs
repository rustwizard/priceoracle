use clap::ArgMatches;

use actix_web::{web, App, HttpResponse, HttpServer};


pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let bind = arg.value_of("bind").unwrap();

    info!(logger, "listening on http://{}", bind);

    let sys = actix_rt::System::new("priceoracle");

    HttpServer::new(|| {
        App::new().route("/", web::get().to(|| HttpResponse::Ok()))
    })
        .bind(bind)
        .unwrap()
        .start();

    let _ = sys.run();

    Ok(())
}