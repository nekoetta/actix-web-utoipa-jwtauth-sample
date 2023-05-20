use jwt_simple::prelude::*;

fn main() {
    let key = HS256Key::generate().to_bytes();
    let hex = key.into_iter().map(|key| {
        format!("{:0>2X}", key)
    }).collect::<Vec<String>>().join(" ");
    println!("{}", hex);
}
