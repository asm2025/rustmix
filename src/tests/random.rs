use rustmix::random;

pub fn test_random() {
    println!("Random string: {}", random::string(10));
    println!("Random char: {}", random::char());
    println!("Random float: {}", random::float());
    println!("Random numeric: {}", random::numeric(1, 10));
    println!("Random boolean: {}", random::boolean());
}
