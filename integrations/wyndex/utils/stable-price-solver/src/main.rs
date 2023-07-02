use std::cmp::Ordering;

use clap::Parser;

/// The maximum x-value to search for
const MAX_X: f64 = 999999999999999999999999999f64;
/// The maximum difference between two values
const PRECISION: f64 = 0.0000000000000001f64;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The `A` parameter of the stable swap formula (amplifier)
    #[arg(short, long)]
    amplifier: f64,

    /// The `D` parameter of the stable swap formula
    #[arg(short, long, default_value_t = 1.0)]
    d: f64,

    /// The target rate of the stable swap formula
    #[arg(short, long, default_value_t = 1.0)]
    target_rate: f64,

    /// The price to solve for
    #[arg(short, long)]
    price: f64,
}

fn main() {
    let args = Args::parse();

    println!(
        "{}",
        find_x(args.amplifier, args.d, args.target_rate, args.price)
    );
}

fn find_x(a: f64, d: f64, e: f64, price: f64) -> f64 {
    binary_search(0.0, MAX_X, price, |x| stable_swap_price(a, d, e, x))
}

/// Calculate the formula of the first derivative of the stable swap formula, negated.
/// Semantically that means that the result is the price of the stable swap at the given x.
///
/// p(x) = -(-2 E x sqrt(E x (4 A D^3 + E x (-4 A D + 4 A E x + D)^2)) - 8 A D E^2 x^2 + 8 A E^3 x^3 - D^3 + 2 D E^2 x^2)/(4 x sqrt(E x (4 A D^3 + E x (-4 A D + 4 A E x + D)^2)))
fn stable_swap_price(a: f64, d: f64, e: f64, x: f64) -> f64 {
    assert!(a > 0.0);
    assert!(d > 0.0);
    assert!(e > 0.0);
    assert!(x > 0.0);
    -(-2f64
        * e
        * x
        * f64::sqrt(
            e * x * (4f64 * a * d.powi(3) + e * x * (-4f64 * a * d + 4f64 * a * e * x + d).powi(2)),
        )
        - 8f64 * a * d * (e * x).powi(2)
        + 8f64 * a * (e * x).powi(3)
        - d.powi(3)
        + 2f64 * d * (e * x).powi(2))
        / (4f64
            * x
            * f64::sqrt(
                e * x
                    * (4f64 * a * d.powi(3)
                        + e * x * (-4f64 * a * d + 4f64 * a * e * x + d).powi(2)),
            ))
}

/// A function that does binary search with a minimum and maximum value
/// This assumes that `f` is monotonically *decreasing*.
fn binary_search(mut min: f64, mut max: f64, find: f64, f: impl Fn(f64) -> f64) -> f64 {
    let mut half = max / 2.0 + min / 2.0;

    let mut last = f64::NAN;
    let mut current = f(half);

    while min <= max {
        let diff = (current - find).abs();
        if diff < PRECISION || last == current {
            return half;
        }
        match current.partial_cmp(&find) {
            Some(Ordering::Greater) => min = half,
            Some(Ordering::Less) => max = half,
            _ => return half,
        }
        half = max / 2.0 + min / 2.0;
        last = current;
        current = f(half);
    }
    half
}

#[cfg(test)]
macro_rules! assert_f64_eq {
    ($a: expr, $b: expr) => {
        let a = $a;
        let b = $b;
        assert!((a - b).abs() < 0.00000000001, "{} != {}", a, b)
    };
}

#[test]
fn price_is_one_at_middle() {
    for a in [0.1, 1.0, 1.5, 2.0, 10.0, 100.0] {
        for d in [0.1, 1.0, 1.5, 2.0, 10.0, 100.0] {
            assert_f64_eq!(stable_swap_price(a, d, 1.0, d as f64 / 2.0), 1.0);
        }
    }
}

#[test]
fn find_x_correct() {
    for a in [1.0, 1.5, 2.0, 10.0, 100.0] {
        for d in [1.0, 1.5, 2.0, 10.0, 100.0] {
            for e in [1.0, 1.5, 2.0, 10.0, 100.0] {
                for price in [0.1, 0.5, 0.9, 1.0, 1.1, 1.5, 1.9, 2.0] {
                    let x = find_x(a, d, e, price);
                    assert_f64_eq!(stable_swap_price(a, d, e, x), price);
                }
            }
        }
    }
}
