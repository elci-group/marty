use warp::Filter;
use crate::memory::{Hotspots, Beliefs, Trace};
use std::sync::Arc;
use parking_lot::Mutex;
use serde_json::json;
use crate::metrics::{self, HTTP_REQUESTS_TOTAL};
use crate::config::SETTINGS;

use crate::outputs;

pub async fn run_server(hotspots: Arc<Mutex<Hotspots>>, beliefs: Arc<Mutex<Beliefs>>, trace: Arc<Mutex<Trace>>) {
    let hs = warp::any().map(move || {
        HTTP_REQUESTS_TOTAL.inc();
        let h = hotspots.lock();
        let v: Vec<_> = h.items.lock().values().map(|hs| {
            json!({"path": hs.path, "energy": hs.energy, "last_ts": hs.last_ts})
        }).collect();
        warp::reply::json(&v)
    });

    let bl = warp::any().map(move || {
        HTTP_REQUESTS_TOTAL.inc();
        let b = beliefs.lock();
        let v: Vec<_> = b.nodes.values().collect::<Vec<_>>();
        warp::reply::json(&v)
    });

    let tr = warp::any().map(move || {
        HTTP_REQUESTS_TOTAL.inc();
        let t = trace.lock();
        warp::reply::json(&t.entries)
    });

    let api_prefix = warp::path!("api" / "v1");

    let port = SETTINGS.read().unwrap().server_port;

    let hotspots_route = api_prefix.and(warp::path("hotspots")).and(hs);
    let beliefs_route = api_prefix.and(warp::path("beliefs")).and(bl);
    let trace_route = api_prefix.and(warp::path("trace")).and(tr);

    let health_route = warp::path("health").map(|| {
        HTTP_REQUESTS_TOTAL.inc();
        warp::reply::json(&json!({"status": "ok"}))
    });

    let metrics_route = warp::path("metrics").map(|| {
        HTTP_REQUESTS_TOTAL.inc();
        metrics::gather_metrics()
    });

    let routes = hotspots_route
        .or(beliefs_route)
        .or(trace_route)
        .or(health_route)
        .or(metrics_route);

    outputs::success(&format!("HTTP dashboard running on http://127.0.0.1:{}", port));
    warp::serve(routes).run(([127,0,0,1], port)).await;
}

