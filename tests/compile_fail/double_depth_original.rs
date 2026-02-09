macro_rules! macro_a_to_end {
    ("a" $body:expr) => {
        $body
    };
}

macro_rules! macro_b_to_a {
    ("b" $body:expr) => {
        "a"
    };
}

fn main() {
    let _ = macro_a_to_end!(macro_b_to_a!("b" "q") "z");
}
