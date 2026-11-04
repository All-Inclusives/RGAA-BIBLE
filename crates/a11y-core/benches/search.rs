use std::time::Instant;

use a11y_core::{A11yCore, ArticleQuery};

fn main() {
    let core = A11yCore::from_bundled().expect("bundled RGAA dataset should be valid");
    let query = ArticleQuery {
        text: Some("alternative textuelle".to_owned()),
        ..ArticleQuery::default()
    };
    let iterations = 5_000;

    let started = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(core.query_articles(&query));
    }

    let elapsed = started.elapsed();
    println!(
        "{iterations} searches in {elapsed:?} ({:.2} µs/search)",
        elapsed.as_secs_f64() * 1_000_000.0 / f64::from(iterations)
    );
}
