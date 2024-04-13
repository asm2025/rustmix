use rustmix::random;

pub fn test_random() {
    println!("Testing random...");
    println!("string: {}", random::string(10));
    println!("char: {}", random::char());
    println!("float: {}", random::float());
    println!("numeric: {}", random::numeric(1, 10));
    println!("boolean: {}", random::boolean());
    println!("word: {}", random::lorem::word());
}
