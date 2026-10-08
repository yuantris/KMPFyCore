use core_math::Expression;
use core_search::SearchEngine;

fn main() {
    println!("math = {}", Expression::eval("sqrt(9) + 2^3").unwrap());
    let mut search = SearchEngine::new();
    search.add(1, "床前明月光").unwrap();
    search.add(2, "明月几时有").unwrap();
    for item in search.search("明月", 10) {
        println!("{} {} {}", item.id, item.score, item.text);
    }
}
