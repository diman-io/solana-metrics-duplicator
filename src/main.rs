use {
    hyper::{
        service::{make_service_fn, service_fn},
        Body, Method, Request, Response, Server,
    },
    log::*,
    std::{convert::Infallible, time::Duration},
};

mod cli;

async fn handle_request(
    request: Request<Body>,
    solana_metrics_url: String,
    mirror_metrics_url: String,
) -> Result<Response<Body>, Infallible> {
    if request.method() == Method::POST && request.uri().path() == "/write" {
        let path_and_query = request.uri().path_and_query().unwrap().to_string();
        let is_solana = {
            let mut is_solana = false;
            let query = request.uri().query().unwrap_or("");
            for pair in query.split('&') {
                let nv: Vec<_> = pair.split('=').collect();
                if nv.len() != 2 {
                    break;
                }
                if nv[0] == "db" {
                    if nv[1] == "tds" || nv[1] == "mainnet-beta" {
                        is_solana = true;
                    }
                    break;
                }
            }
            is_solana
        };
        let body_bytes = hyper::body::to_bytes(request.into_body()).await.unwrap();
        let url = if is_solana {
            let body_bytes = body_bytes.clone();
            let url = mirror_metrics_url.clone() + &path_and_query;
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client.post(url).body(body_bytes).send().await;
            });
            solana_metrics_url+ &path_and_query
        } else {
            mirror_metrics_url + &path_and_query
        };
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let solana_response = client.post(url).body(body_bytes).send();

        let response = if let Ok(solana_response) = solana_response {
            Response::builder()
                .status(solana_response.status().as_u16())
                .body(Body::from(solana_response.bytes().unwrap()))
                .unwrap()
        } else {
            Response::builder()
                .status(500)
                .header("Content-Type", "text/plain")
                .body(Body::from(format!("{}", solana_response.unwrap_err())))
                .unwrap()
        };
        return Ok(response);
    }

    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/plain")
        .body(Body::from("Not Found"))
        .unwrap())
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp_micros().init();
    let args = cli::Args::parse();
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
