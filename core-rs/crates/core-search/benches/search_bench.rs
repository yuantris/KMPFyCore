use core_search::SearchEngine;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let mut engine = SearchEngine::new();
    for id in 0..100_000u64 {
        engine.add(id, &format!("床前明月光 疑是地上霜 {id}")).unwrap();
    }

    let start = Instant::now();
    let result = engine.search(black_box("明月"), 20);
    let elapsed = start.elapsed();
    println!("results={}, elapsed={elapsed:?}", result.len());
}
