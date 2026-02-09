use inside_out_expand::inside_out_expand;

macro_rules! macro_a_to_end {
    ("a" $body:expr) => {
        $body
    };
}

macro_rules! macro_nonlit_out {
    ($body:expr) => {
        {
            const DEFINED_IN_MACRO: &str = $body;
            DEFINED_IN_MACRO
        }
    };
}

fn main() {
    let _ = inside_out_expand!(macro_nonlit_out!(macro_a_to_end!("a" "q")));
}
