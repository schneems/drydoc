#[derive(Debug)]
struct Person {
    name: String,
}

impl Person {
    fn new() -> Self {
        Self {
            name: "{{name|default('Richard')}}".to_string(),
        }
    }
}
