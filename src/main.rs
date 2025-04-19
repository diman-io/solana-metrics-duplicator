use {
    hyper::{
        service::{make_service_fn, service_fn},
        Body, Method, Request, Response, Server, StatusCode,
    },
    log::*,
    std::{convert::Infallible, time::Duration},
};

mod cli;

async fn handle_request(
    request: Request<Body>,
    solana_metrics_url: Option<String>,
    mirror_metrics_url: Option<String>,
) -> Result<Response<Body>, Infallible> {
    if !(request.method() == Method::POST && request.uri().path() == "/write") {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap());
    };

    let path_and_query = request.uri().path_and_query().unwrap().to_string();
    let is_solana_db = {
        let mut is_solana_db = false;
        let query = request.uri().query().unwrap_or("");
        for pair in query.split('&') {
            let nv: Vec<_> = pair.split('=').collect();
            if nv.len() != 2 {
                break;
            }
            if nv[0] == "db" {
                if nv[1] == "tds" || nv[1] == "mainnet-beta" {
                    is_solana_db = true;
                }
                break;
            }
        }
        is_solana_db
    };

    // is_solana_db     + - + - + -
    // is_solana_url    + + - - + +
    // is_mirror_url    + + + + - -
    // send_to_solana   S - - - S -
    // send_to_mirror   A S S S - -

    let (sync_url, async_url) = if is_solana_db && solana_metrics_url.is_some() {
        (&solana_metrics_url, &mirror_metrics_url)
    } else {
        (&mirror_metrics_url, &None)
    };

    if sync_url.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap());
    }

    let body_bytes = hyper::body::to_bytes(request.into_body()).await.unwrap();
    debug!(
        "{}",
        match std::str::from_utf8(&body_bytes) {
            Ok(s) => format!("{}\n{}", path_and_query, s),
            Err(e) => format!("ERROR: {}\n{}\n{:?}", e, path_and_query, body_bytes),
        }
    );

    if async_url.is_some() {
        let body_bytes = body_bytes.clone();
        let url = format!("{}{}", async_url.as_ref().unwrap(), path_and_query);
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let _ = client.post(url).body(body_bytes).send().await;
        });
    }

    let url = format!("{}{}", sync_url.as_ref().unwrap(), path_and_query);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let upstream = client
        .post(url)
        .body(body_bytes)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
    let mut response_builder = Response::builder().status(upstream.status());
    for (key, value) in upstream.headers().iter() {
        response_builder = response_builder.header(key, value);
    }
    Ok(response_builder
        .body(Body::from(upstream.bytes().unwrap()))
        .unwrap())
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp_micros().init();
    let args = cli::Args::parse();
    assert!(args.solana_metrics_url.is_some() || args.mirror_metrics_url.is_some());
    let service = make_service_fn(move |_| {
        let solana_metrics_url = args.solana_metrics_url.clone();
        let mirror_metrics_url = args.mirror_metrics_url.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                handle_request(
                    request,
                    solana_metrics_url.clone(),
                    mirror_metrics_url.clone(),
                )
            }))
        }
    });

    let addr = args.bind_addr.into();
    let server = Server::bind(&addr).serve(service);
    info!("Server listening on http://{addr}");

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
