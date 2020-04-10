use clap::ArgMatches;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

async fn routes(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to / such as: `curl localhost:8000/ -XPOST -d 'hello'`",
        ))),
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let bind = arg.value_of("bind").unwrap();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(routes)) });

    let server = Server::bind(bind.unwrap()).serve(service);
    info!(logger, "listening on http://{}", bind);
    server.await?;
    Ok(())
}
