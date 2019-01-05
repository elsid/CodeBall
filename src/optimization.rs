pub fn minimize1d<F>(mut begin: f64, mut end: f64, iterations: usize, function: F) -> f64
    where F: Fn(f64) -> f64
{
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 0.0;
    let mut y2 = 0.0;
    let mut is1 = false;
    let mut is2 = false;
    let inverted_phi = 2.0 / (1.0 + 5.0f64.sqrt());
    (0..iterations)
        .for_each(|_| {
            if !is1 {
                x1 = end - (end  - begin) * inverted_phi;
                y1 = function(x1);
                is1 = true;
            }
            if !is2 {
                x2 = begin + (end - begin) * inverted_phi;
                y2 = function(x2);
                is2 = true;
            }
            if y1 < y2 {
                end = x2;
                x2 = x1;
                y2 = y1;
                is1 = false;
            } else {
                begin = x1;
                x1 = x2;
                y1 = y2;
                is2 = false;
            }
        });
    (begin + end) / 2.0
}

pub fn minimize2d<F>(initial: &(f64, f64), function_calls: usize, function: F) -> (f64, f64)
    where F: Fn(f64, f64) -> f64 {
    use ndarray::prelude::{Array, ArrayView1};
    use optimize::{NelderMeadBuilder, Minimizer};

    let args = Array::from_vec(vec![
        initial.0,
        initial.1,
    ]);

    let optimizer = NelderMeadBuilder::default()
        .maxfun(function_calls)
        .build()
        .unwrap();

    let result = optimizer.minimize(
        |x: ArrayView1<f64>| function(x[0], x[1]),
        args.view()
    );

    (result[0], result[1])
}
