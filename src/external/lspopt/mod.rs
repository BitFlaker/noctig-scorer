use std::collections::HashMap;
use std::sync::LazyLock;
use std::io::Cursor;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ndarray::{Array1, Array2, Axis, Zip};
use ndarray_npy::ReadNpyExt;

use crate::external::scipy::{spectrogram, Spectrogram};

static C: LazyLock<Array1<f64>> = LazyLock::new(|| {
    let reader = Cursor::new(include_bytes!("data/c.npy"));
    let c: Array2<f64> = ReadNpyExt::read_npy(reader).unwrap();
    c.row(0).to_owned()
});

static WEIGHTS: LazyLock<Array2<f64>> = LazyLock::new(|| {
    let reader = Cursor::new(include_bytes!("data/weights.npy"));
    let c: Array2<f64> = ReadNpyExt::read_npy(reader).unwrap();
    c
});

const K_TO_VALUE: LazyLock<HashMap<usize, f64>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, 5.4);
    map.insert(2, 6.0);
    map.insert(3, 7.3);
    map.insert(4, 8.1);
    map.insert(5, 8.7);
    map.insert(6, 9.3);
    map.insert(7, 9.8);
    map.insert(8, 10.3);
    map.insert(9, 10.9);
    map.insert(10, 11.2);
    map
});

pub fn spectrogram_lspopt(
    data: Array1<f64>,
    sf: f64,
    nperseg: i32,
) -> Spectrogram {
    let c_parameter = 20.0;
    let (h, taper_weights) = lspopt(nperseg, c_parameter);

    Zip::from(h.rows())
        .and(&taper_weights)
        .into_par_iter()
        .map(|(taper_window, taper_weight)| {
            let mut spec = spectrogram(&data, taper_window, sf);
            spec.result *= *taper_weight;
            spec
        })
        .reduce_with(|mut acc, spec| {
            acc.freqs = spec.freqs;
            acc.time = spec.time;
            acc.result += &spec.result;
            acc
        })
        .unwrap()
}

pub fn lspopt(n: i32, c_parameter: f64) -> (Array2<f64>, Array1<f64>) {
    let k = ((c_parameter - 1.0) * 10.0).round() as usize;
    if c_parameter != C[k] {
        println!("Using c={} instead of desired {}", C[k], c_parameter)
    }

    let weights: Array1<f64> = WEIGHTS
        .column(k)
        .iter()
        .filter(|&&v| v != 0.0)
        .take(10)
        .copied()
        .collect();

    let sum = weights.sum();
    let weights = weights / sum;
    let k = weights.len();

    let t1 = arange_like(n) / f_h(n, k);
    let mut h = ndarray::stack(Axis(0),
    &[
        Array1::ones(n as usize).view(),
        (2.0 * &t1).view(),
    ]).unwrap();

    for i in 1..(k - 1) {
        let hi = &h.index_axis(Axis(0), i);
        let hi_prev = &h.index_axis(Axis(0), i - 1);
        let new_col = 2.0 * &t1 * hi - (2 * i) as f64 * hi_prev;

        h = ndarray::concatenate(Axis(0),
        &[
            h.view(),
            new_col.insert_axis(Axis(0)).view()
        ]).unwrap();
    }

    let exp_term = t1.mapv(|x| (-x.powi(2) / 2.0).exp());
    let outer_product = exp_term.insert_axis(Axis(1)).dot(&Array1::ones(k).insert_axis(Axis(0)));
    h = h.t().to_owned() * &outer_product;

    let norms: Array1<f64> = h.axis_iter(Axis(1))
        .map(|col| col.dot(&col).sqrt())
        .collect();
    h = h / &norms.insert_axis(Axis(0));

    (h.t().to_owned(), weights)
}

fn arange_like(n: i32) -> Array1<f64> {
    let start = -(n / 2) + 1;
    let end =  n / 2;
    Array1::from_iter((start..=end).map(|x| x as f64))
}

fn f_h(n: i32, k: usize) -> f64 {
    n as f64 / K_TO_VALUE.get(&k).unwrap_or(&1.0)
}
