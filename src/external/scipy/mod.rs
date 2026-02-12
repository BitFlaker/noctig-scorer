use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis, s};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use realfft::num_complex::Complex64;
use realfft::RealFftPlanner;

#[derive(Clone)]
pub struct Spectrogram {
    pub freqs: Array1<f64>,
    pub time: Array1<f64>,
    pub result: Array2<f64>
}

/// Always assumes the to perform on the last axis, noverlap = 0
pub fn spectrogram(x: &Array1<f64>, window: ArrayView1<f64>, fs: f64) -> Spectrogram {
    let nperseg = window.len();

    let nfft = nperseg;
    let noverlap = 0;   // Has to be 0, otherwise requires sliding window below and further adjustments
    let step = nperseg - noverlap;

    let scale = 1.0 / (fs * (&window * &window).sum());
    let freqs = rfftfreq(nfft, 1.0 / fs);

    // Implementation of `result = _fft_helper(...)``

    // Detrend and apply window (Due to `noverlap` being 0, no sliding window is required)
    let result = crate::views::spectrogram::utils::chunked_window_view(x, nperseg);
    let detrended = detrend_constant(&result);
    let mut result = &detrended * &window;

    // Create the FFT planner
    let mut real_planner = RealFftPlanner::<f64>::new();
    let length = result.len_of(Axis(1));
    let r2c = real_planner.plan_fft_forward(length);

    // Get the output shape
    let rows = result.shape()[0];
    let cols = result.shape()[1] / 2 + 1;

    // Create a default complex buffer which will be fully overwritten later on
    let mut result_complex = Array2::<Complex64>::from_shape_vec((rows, cols), vec![Complex64::default(); rows * cols]).unwrap();

    // Get parallel input and output chunks
    let real_slice = result.as_slice_mut().unwrap().par_chunks_exact_mut(length);
    let complex_slice = result_complex.as_slice_mut().unwrap().par_chunks_exact_mut(cols);

    // Perform the FFT
    real_slice
        .zip(complex_slice)
        .for_each(|(row, res)| {
            r2c.process(row, res).unwrap();
        });

    result_complex.mapv_inplace(|x| (x.conj() * x) * scale);
    if nfft % 2 != 0 {
        result_complex.slice_mut(s![.., 1..]).mapv_inplace(|x| x * 2.0);
    } else {
        result_complex.slice_mut(s![.., 1..-1]).mapv_inplace(|x| x * 2.0);
    }

    let start = nperseg / 2;
    let stop = x.len() - nperseg / 2 + 1;
    let time = Array1::from_iter(
        (start..stop)
            .step_by(step)
            .map(|x| x as f64 / fs as f64)
    );

    let result = result_complex.t().mapv(|x| x.re);

    Spectrogram {
        freqs,
        time,
        result
    }
}

fn rfftfreq(n: usize, d: f64) -> Array1<f64> {
    let val = 1.0 / (n as f64 * d);
    let n = n / 2 + 1;
    let results = Array1::from_iter((0..n).map(|x| x as f64));
    return results * val
}

fn detrend_constant(array: &ArrayView2<f64>) -> Array2<f64> {
    array - array.mean_axis(Axis(1)).unwrap().insert_axis(Axis(1))
}
