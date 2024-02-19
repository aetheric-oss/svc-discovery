//! Example communication with this service

use hyper::client::connect::HttpConnector;
use hyper::{Body, Client, Method, Request, Response};
use hyper::{Error, StatusCode};
use lib_common::grpc::get_endpoint_from_env;
use svc_discovery_client_rest::types::*;

fn evaluate(resp: Result<Response<Body>, Error>, expected_code: StatusCode) -> (bool, String) {
    let mut ok = true;
    let result_str: String = match resp {
        Ok(r) => {
            let tmp = r.status() == expected_code;
            ok &= tmp;
            println!("{:?}", r.body());

            r.status().to_string()
        }
        Err(e) => {
            ok = false;
            e.to_string()
        }
    };

    (ok, result_str)
}

async fn uss(
    url: &str,
    client: &mut Client<HttpConnector>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ok = true;

    // GET /uss/get_flights
    {
        let data = GetFlightsRequest {
            view: "0.0,0.0,0.0,0.0".to_string(),
            recent_positions_duration: 10.,
        };

        let uri = format!(
            "{}/uss/flights?view={}&recent_positions_duration={}",
            url, data.view, data.recent_positions_duration
        );

        let req = Request::builder()
            .method(Method::GET)
            .uri(uri.clone())
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let resp = client.request(req).await;
        let (success, result_str) = evaluate(resp, StatusCode::OK);
        ok &= success;

        println!("{}: {}", uri, result_str);
    }

    Ok(ok)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NOTE: Ensure the server is running, or this example will fail.");

    let (host, port) = get_endpoint_from_env("SERVER_HOSTNAME", "SERVER_PORT_REST");
    let url = format!("http://{host}:{port}");
    let mut ok = true;
    let mut client = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(10))
        .build_http();

    ok &= uss(&url, &mut client).await?;

    if ok {
        println!("\u{1F9c1} All endpoints responded!");
    } else {
        eprintln!("\u{2620} Errors");
    }

    Ok(())
}
