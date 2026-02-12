use ndarray::{Array1, Array2, ArrayView2, ShapeBuilder};

pub fn chunked_window_view<'a, T>(x: &'a Array1<T>, window: usize) -> ArrayView2<'a, T> {
    let n_windows = (x.len() - window) / window + 1;
    let shape = (n_windows, window).strides((window, 1));
    ArrayView2::from_shape(shape, x.as_slice().unwrap()).unwrap()
}

pub fn calculate_percentiles(array2d: &Array2<f64>, trimperc: f64) -> (f64, f64) {
    let q_min = trimperc / 100.0;
    let q_max = 1.0 - q_min;

    // Flatten and sort the data
    let mut data: Vec<f64> = array2d.iter().copied().collect();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Calculate percentile indices
    let min = percentile_linear(&data, q_min);
    let max = percentile_linear(&data, q_max);

    (min, max)
}

fn percentile_linear(sorted_data: &[f64], quantile: f64) -> f64 {
    let idx = (sorted_data.len() - 1) as f64 * quantile;
    let lower_idx = idx.floor() as usize;
    let upper_idx = idx.ceil() as usize;

    if lower_idx == upper_idx {
        sorted_data[lower_idx]
    } else {
        let lower_val = sorted_data[lower_idx];
        let upper_val = sorted_data[upper_idx];
        let fraction = idx - lower_idx as f64;

        lower_val + fraction * (upper_val - lower_val)
    }
}
