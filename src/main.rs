use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora_core::prelude::*;
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_limits::rate::Rate;
use pingora_load_balancing::prelude::{RoundRobin, TcpHealthCheck};
use pingora_load_balancing::LoadBalancer;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();
    let mut upstreams = LoadBalancer::try_from_iter(["127.0.0.1:8088", "127.0.0.1:9099"]).unwrap();

    //Setup healcheck
    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(Duration::from_secs(1));
    // Set background service
    let background = background_service("health check", upstreams);
    let upstreams = background.task();

    // Set load balancer
    let mut lb = http_proxy_service(&server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188");

    // let rate = Rate
    server.add_service(background);
    server.add_service(lb);
    server.run_forever();
}
