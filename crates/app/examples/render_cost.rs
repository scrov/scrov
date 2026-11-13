//! Prints how long the router takes per request: `cargo run --release -p scrov-app --example render_cost`.
use scrov_app::{handle, Request};
use std::time::Instant;

fn main() {
    for path in [
        "/",
        "/blog/",
        "/blog/first-post/",
        "/lab/",
        "/llms.txt",
        "/fragments/project/0",
    ] {
        let request = Request::get(path);
        let start = Instant::now();
        let first = handle(&request);
        let cold = start.elapsed();
        let runs = 200;
        let start = Instant::now();
        for _ in 0..runs {
            std::hint::black_box(handle(&request));
        }
        let warm = start.elapsed() / runs;
        println!(
            "{path:<24} {:>7} bytes  first {cold:>10.1?}  then {warm:>10.1?}",
            first.body.len()
        );
    }
}
