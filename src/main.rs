use std::str::FromStr;

use num::Complex;
fn main() {
    println!("Hello, world!");
}

fn parse_pair<T: FromStr>(s: &str, seperator: char) -> Option<T, T>{
    
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}
