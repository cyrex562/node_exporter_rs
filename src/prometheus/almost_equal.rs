// Copyright (c) 2015 Björn Rabenstein
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// The code in this package is copy/paste to avoid a dependency. Hence this file
// carries the copyright of the original repo.
// https://github.com/beorn7/floats

use std::f64;

const MIN_NORMAL_FLOAT64: f64 = f64::MIN_POSITIVE;

// Returns true if `a` and `b` are equal within a relative error of `epsilon`.
// See http://floating-point-gui.de/errors/comparison/ for the details of the applied method.
pub fn almost_equal_float64(a: f64, b: f64, epsilon: f64) -> bool {
    if a == b {
        return true;
    }
    let abs_a = a.abs();
    let abs_b = b.abs();
    let diff = (a - b).abs();
    if a == 0.0 || b == 0.0 || abs_a + abs_b < MIN_NORMAL_FLOAT64 {
        return diff < epsilon * MIN_NORMAL_FLOAT64;
    }
    diff / abs_a.min(abs_b).min(f64::MAX) < epsilon
}

// The slice form of `almost_equal_float64`.
pub fn almost_equal_float64s(a: &[f64], b: &[f64], epsilon: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if !almost_equal_float64(a[i], b[i], epsilon) {
            return false;
        }
    }
    true
}

fn main() {
    // Example usage
    let a = 1.0;
    let b = 1.0 + 1e-10;
    let epsilon = 1e-9;
    println!("Almost equal: {}", almost_equal_float64(a, b, epsilon));

    let vec_a = vec![1.0, 2.0, 3.0];
    let vec_b = vec![1.0 + 1e-10, 2.0, 3.0];
    println!("Almost equal slices: {}", almost_equal_float64s(&vec_a, &vec_b, epsilon));
}