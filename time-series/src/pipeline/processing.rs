use anyhow::{Result, anyhow};
use peroxide::fuga::Statistics as _;
use rustfft::{FftDirection, FftPlanner, num_complex::Complex};

use super::{PipelineIntermediate, Record, Signal, StepConfig};

/// Uses welfords online algorithm for numerical stability
pub fn average(data: Signal) -> f64 {
    let mut xn = 0f64;
    let mut n = 0f64;

    for x in data {
        n += 1f64;
        xn += (x.y - xn) / n;
    }
    xn.abs()
}

/// Uses welfords online algorithm for numerical stability
pub fn variance(data: Signal) -> f64 {
    let mut xn = 0f64;
    let mut n = 0f64;
    let mut m2n: f64 = 0f64;

    for x in data {
        n += 1f64;
        let diff_1 = x.y - xn;
        xn += diff_1 / n;
        m2n += diff_1 * (x.y - xn);
    }
    assert_ne!(n, 1f64);
    m2n / (n - 1f64)
}

pub fn rolling_avg(window_size: usize, data: Signal) -> Signal {
    let mut window: Vec<f64> = vec![];
    let mut w = 0;
    let mut out = vec![];

    let mut running_mean = 0.0;

    for x in data {
        if window.len() < window_size {
            window.push(x.y);
            running_mean = window.mean();
        } else {
            running_mean -= window[w] / (window_size as f64);
            window[w] = x.y;
            running_mean += window[w] / (window_size as f64);
        }
        out.push(Record {
            x: x.x,
            y: running_mean,
        });
        w += 1;
        if w >= window.len() {
            w = 0;
        }
    }
    out
}

pub fn smooth_reals(window_size: usize, data: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut window: Vec<f64> = vec![];
    let mut w = 0;
    let mut out = vec![];

    let mut running_mean = 0.0;

    for x in data {
        if window.len() < window_size {
            window.push(x.re);
            running_mean = window.mean();
        } else {
            running_mean -= window[w] / (window_size as f64);
            window[w] = x.re;
            running_mean += window[w] / (window_size as f64);
        }
        out.push(Complex::<f64> {
            re: running_mean,
            im: x.im,
        });
        w += 1;
        if w >= window.len() {
            w = 0;
        }
    }
    out
}

pub fn reals_abs(data: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut data = data.clone();
    for x in &mut data {
        x.re = x.re.abs();
    }
    data
}

pub fn skip_first_complex(data: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    data[1..data.len()].to_vec()
}

pub fn fourier_transform(data: Signal) -> Vec<Complex<f64>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(data.len(), FftDirection::Forward);
    let mut buffer = vec![
        Complex {
            re: 0.0f64,
            im: 0.0f64
        };
        data.len()
    ];
    for i in 0..data.len() {
        buffer[i].re = data[i].y;
    }
    fft.process(&mut buffer);
    buffer
}

/// **OBS**: Assumes the input data is an un-normalized FFT
pub fn inverse_fourier_transform(data: Vec<Complex<f64>>) -> Signal {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(data.len(), FftDirection::Inverse);
    let mut buffer = data.clone();
    let n = data.len() as f64;
    fft.process(&mut buffer);
    let fs = 15625.0; // Sampling frequency

    let mut out = vec![];
    for (i, _) in data.iter().enumerate() {
        let scaled = buffer[i].scale(1.0 / n);
        out.push(Record {
            x: (i as f64) / fs,
            y: scaled.re,
        });
    }
    out
}

pub fn post_fft_formatting(data: Vec<Complex<f64>>) -> Signal {
    let mut out = vec![];
    let n = data.len() as f64;
    let fs = 15625.0; // Sampling frequency
    let delta_f = fs / n; // Frequency resolution
    for (i, val) in data.iter().enumerate() {
        let freq = i as f64 * delta_f;
        let scaled = val.scale(1.0 / n.sqrt());
        out.push(Record {
            x: freq,
            y: scaled.re,
        });
    }
    out
}

pub fn normalize(data: Signal) -> Signal {
    let mut max_i = 0;
    for i in 0..data.len() {
        if data[i].y > data[max_i].y {
            max_i = i;
        }
    }
    let mut out = vec![];
    for x in &data {
        out.push(Record {
            x: x.x,
            y: x.y / data[max_i].y,
        });
    }
    out
}

pub fn skip_first(data: Signal) -> Signal {
    data[1..data.len()].to_vec()
}

pub fn bandpass_filter(mut data: Vec<Complex<f64>>, low: f64, high: f64) -> Vec<Complex<f64>> {
    let frequencies = post_fft_formatting(data.clone());
    for i in 0..data.len() {
        if frequencies[i].x < low || frequencies[i].x > high {
            data[i].re = 0.0;
            data[i].im = 0.0;
        }
    }
    data
}

fn scale_axis(axis: super::AxisSelection, factor: f64, mut vec: Signal) -> Signal {
    for sample in &mut vec {
        match axis {
            super::AxisSelection::X => sample.x *= factor,
            super::AxisSelection::Y => sample.y *= factor,
        }
    }
    vec
}

fn log_axis(axis: super::AxisSelection, base: f64, mut vec: Vec<Record>) -> Vec<Record> {
    for sample in &mut vec {
        match axis {
            super::AxisSelection::X => sample.x = sample.x.log(base),
            super::AxisSelection::Y => sample.y = sample.y.log(base),
        }
    }
    vec
}

// TODO: Return a better error which can be used in the ui
pub fn run_pipeline(
    pipeline: &[StepConfig],
    input: PipelineIntermediate,
) -> Result<PipelineIntermediate> {
    let mut out = input.clone();
    for step in pipeline {
        out = match out {
            PipelineIntermediate::DataFrame(df) => match step {
                StepConfig::PickColumns { column_1, column_2 } => {
                    PipelineIntermediate::Signal(df.pick(*column_1, *column_2))
                }
                _ => return Err(anyhow!("The pipeline expected a dataframe at this step.")),
            },
            PipelineIntermediate::Signal(vec) => match step {
                StepConfig::Average => PipelineIntermediate::Signal(vec![Record {
                    x: 0.0,
                    y: average(vec),
                }]),
                StepConfig::Variance => PipelineIntermediate::Signal(vec![Record {
                    x: 0.0,
                    y: variance(vec),
                }]),
                StepConfig::SmoothSignal { window } => {
                    PipelineIntermediate::Signal(rolling_avg(*window, vec))
                }
                StepConfig::FourierTransform => {
                    PipelineIntermediate::Complex(fourier_transform(vec))
                }
                StepConfig::SkipFirstEntry => PipelineIntermediate::Signal(skip_first(vec)),
                StepConfig::Normalize => PipelineIntermediate::Signal(normalize(vec)),
                StepConfig::ScaleAxis { axis, factor } => {
                    PipelineIntermediate::Signal(scale_axis(*axis, *factor, vec))
                }
                StepConfig::LogAxis { axis, base } => {
                    PipelineIntermediate::Signal(log_axis(*axis, *base, vec))
                }
                _ => return Err(anyhow!("The pipeline didn't expect a signal at this step.")),
            },
            PipelineIntermediate::Complex(vec) => match step {
                StepConfig::SmoothReals { window } => {
                    PipelineIntermediate::Complex(smooth_reals(*window, vec))
                }
                StepConfig::AbsoluteValueOfReals => PipelineIntermediate::Complex(reals_abs(vec)),
                StepConfig::InverseFourierTransform => {
                    PipelineIntermediate::Signal(inverse_fourier_transform(vec))
                }
                StepConfig::PostFFTFormatting => {
                    PipelineIntermediate::Signal(post_fft_formatting(vec))
                }
                StepConfig::SkipFirstComplexEntry => {
                    PipelineIntermediate::Complex(skip_first_complex(vec))
                }
                StepConfig::BandpassFilter { middle, half_width } => PipelineIntermediate::Complex(
                    bandpass_filter(vec, middle - half_width, middle + half_width),
                ),
                _ => {
                    return Err(anyhow!(
                        "The pipeline didn't expect a complex series at this step."
                    ));
                }
            },
        }
    }
    Ok(out)
}

pub fn slope(r1: Record, r2: Record) -> f64 {
    (r2.y - r1.y) / (r2.x - r1.x)
}

/// Returns the current in amperes
///
/// # Arguments
///
/// * `capacitance`   - The capacitance in Farads (SI unit)
/// * `voltage_slope` - U / dt
pub fn current(capacitance: f64, voltage_slope: f64) -> f64 {
    capacitance * voltage_slope
}

/// Returns the energy difference stored in a cacapitor in Joules
///
/// # Arguments
///
/// * `capacitance`   - The capacitance in Farads (SI unit)
/// * `starting_voltage` - The voltage before charging in Volts
/// * `end_voltage` - The voltage after charging in Volts
pub fn energy(capacitance: f64, starting_voltage: f64, end_voltage: f64) -> f64 {
    0.5 * capacitance * (end_voltage.powi(2) - starting_voltage.powi(2))
}
