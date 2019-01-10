#![allow(unused_variables)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use super::statistics;

use serde_derive::Serialize;

use actix;
use bytes::Bytes;
use env_logger;
use futures::sync::mpsc;
use futures::Stream;

use actix_web::http::{header, Method, StatusCode};
use actix_web::middleware::session::{self, RequestSession};
use actix_web::{
    error, fs, middleware, pred, server, App, Error, FutureResponse, HttpRequest, HttpResponse,
    Path, Result, Json
};
use futures::future::{result, Future, FutureResult};
use std::{env, io};

struct AppState {
    tx: crossbeam_channel::Sender<statistics::Message>,
}

fn fnonce_to_fn<T, A>(func: T) -> Box<dyn FnMut(A) + Send + 'static>
where
    T: FnOnce(A) + Send + 'static,
{
    let mut foo = Some(func);
    Box::new(move |a: A| {
        (foo.take().unwrap())(a);
    })
}

/// favicon handler
fn favicon(req: &HttpRequest<AppState>) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

/// simple index handler
fn welcome(req: &HttpRequest<AppState>) -> Result<HttpResponse> {
    println!("{:?}", req);

    // session
    let mut counter = 1;
    if let Some(count) = req.session().get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        counter = count + 1;
        req.session().set("counter", counter)?;
    } else {
        req.session().set("counter", counter)?;
    }

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/welcome.html")))
}

/// 404 handler
fn p404(req: &HttpRequest<AppState>) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

/// async handler
fn index_async(req: &HttpRequest<AppState>) -> FutureResult<HttpResponse, Error> {
    println!("{:?}", req);

    result(Ok(HttpResponse::Ok().content_type("text/html").body(
        format!("Hello {}!", req.match_info().get("name").unwrap()),
    )))
}

fn api_async(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    //println!("{:?}", req);

    let state = req.state();

    let (sender, receiver) = futures::sync::oneshot::channel::<statistics::Bitrates>();

    let callback = fnonce_to_fn(move |s:statistics::Bitrates| {
        sender.send(s);
    });
    state.tx.send(statistics::Message::GetBitrate(callback));

    #[derive(Serialize)]
    struct Object {
        avg_bytes_per_second: u64,
        last_second_bytes: u64,
    }

    Box::new(
        receiver
            .map_err(Error::from)
            .map(|bitrates| Object { avg_bytes_per_second: bitrates.avg_bytes_per_second, last_second_bytes: bitrates.seconds_rates[1] })
            .map(|s| HttpResponse::Ok().json(s)),
    )
}

/// async body
fn index_async_body(path: Path<String>) -> HttpResponse {
    let text = format!("Hello {}!", *path);

    let (tx, rx_body) = mpsc::unbounded();
    let _ = tx.unbounded_send(Bytes::from(text.as_bytes()));

    HttpResponse::Ok().streaming(rx_body.map_err(|e| error::ErrorBadRequest("bad request")))
}

/// handler with path parameters like `/user/{name}/`
fn with_param(req: &HttpRequest<AppState>) -> HttpResponse {
    println!("{:?}", req);

    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Hello {}!", req.match_info().get("name").unwrap()))
}

pub fn thread(tx: crossbeam_channel::Sender<statistics::Message>) {
    env::set_var("RUST_LOG", "actix_web=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let sys = actix::System::new("basic-example");

    let addr = server::new(move || {
        let state = AppState { tx: tx.clone() };
        App::with_state(state)
            // enable logger
            .middleware(middleware::Logger::default())
            // cookie session middleware
            .middleware(session::SessionStorage::new(
                session::CookieSessionBackend::signed(&[0; 32]).secure(false),
            ))
            .resource("/api", |r| r.method(Method::GET).a(api_async))
            // register favicon
            .resource("/favicon", |r| r.f(favicon))
            // register simple route, handle all methods
            .resource("/welcome", |r| r.f(welcome))
            // with path parameters
            .resource("/user/{name}", |r| r.method(Method::GET).f(with_param))
            // async handler
            .resource("/async/{name}", |r| r.method(Method::GET).a(index_async))
            // async handler
            .resource("/async-body/{name}", |r| {
                r.method(Method::GET).with(index_async_body)
            })
            .resource("/test", |r| {
                r.f(|req| match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    Method::POST => HttpResponse::MethodNotAllowed(),
                    _ => HttpResponse::NotFound(),
                })
            })
            .resource("/error", |r| {
                r.f(|req| {
                    error::InternalError::new(
                        io::Error::new(io::ErrorKind::Other, "test"),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })
            })
            // static files
            
            // redirect
            .resource("/", |r| {
                r.method(Method::GET).f(|req| {
                    println!("{:?}", req);
                    HttpResponse::Found()
                        .header(header::LOCATION, "index.html")
                        .finish()
                })
            })
            .handler("/", fs::StaticFiles::new("static").unwrap())
            // default
            .default_resource(|r| {
                // 404 for GET request
                r.method(Method::GET).f(p404);

                // all requests that are not `GET`
                r.route()
                    .filter(pred::Not(pred::Get()))
                    .f(|req| HttpResponse::MethodNotAllowed());
            })
    })
    .bind("10.0.0.1:8080")
    .and_then(|s| s.bind("192.168.4.1:8080"))
    .expect("Can not bind to 10.0.0.1:8080 or 192.168.4.1:8080")
    .shutdown_timeout(0) // <- Set shutdown timeout to 0 seconds (default 60s)
    .start();

    println!("Starting http server: 10.0.0.1:8080");
    let _ = sys.run();
}
