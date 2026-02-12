use crate::views::spectrogram::utils::calculate_percentiles;
use crate::external::scipy::Spectrogram;

/// Transforms the given spectrogram to dB/Hz across hours.
/// This functions is based on the implementation [here](https://github.com/raphaelvallat/yasa/blob/c18f1a6561fea6e31613fd336568b50466a7f03e/src/yasa/plotting.py#L170)
pub fn plot_spectrogram(mut spectrogram: Spectrogram) -> (Spectrogram, (f64, f64)) {
    let trimperc = 2.5;
    let fmin = 0.5;
    let fmax = 25.0;

    let indices: Vec<usize> = spectrogram.freqs
        .mapv(|x| x >= fmin && x <= fmax)
        .iter()
        .enumerate()
        .filter_map(|(i, &keep)| if keep { Some(i) } else { None })
        .collect();

    // Convert uV^2 / Hz --> dB / Hz
    spectrogram.result.mapv_inplace(|x| 10.0 * x.log10());

    // Only take relevant frequencies (based on fmin / fmax above)
    spectrogram.result = spectrogram.result.select(ndarray::Axis(0), &indices);
    spectrogram.freqs = spectrogram.freqs.select(ndarray::Axis(0), &indices);

    // Convert time to hours
    spectrogram.time.mapv_inplace(|x| x / 3600.0);

    // Calculate lower and upper range of color scale
    let limits = calculate_percentiles(&spectrogram.result, trimperc);

    (spectrogram, limits)
}
