use anyhow::{Result, anyhow};
use rustfft::num_complex::Complex;
use serde::{Deserialize, Serialize};
use std::{fs::File, hash::Hash, io::BufReader, path::Path};
use strum::{Display, EnumString};

pub mod processing;
pub mod ui;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Record {
    pub x: f64,
    pub y: f64,
}

impl Hash for Record {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_le_bytes().hash(state);
        self.y.to_le_bytes().hash(state);
    }
}

impl Record {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataFrame {
    pub column_names: Vec<String>,
    pub columns: Vec<Vec<f64>>,
}

impl DataFrame {
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|_| anyhow!("Error opening file"))?;
        let reader = BufReader::new(file);
        let mut rdr = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(reader);
        let mut out = DataFrame::default();
        for h in rdr.headers()? {
            out.column_names.push(h.to_string());
            out.columns.push(vec![]);
        }
        for record in rdr.records().flatten() {
            for (i, r) in record.iter().enumerate() {
                match r.parse() {
                    Ok(f) => {
                        out.columns[i].push(f);
                    }
                    Err(_) => {}
                }
            }
        }

        let mut empty_columns = vec![];
        for (i, col) in out.columns.iter().enumerate() {
            if col.is_empty() {
                empty_columns.push(i);
            }
        }
        for idx in empty_columns.iter().rev() {
            out.columns.remove(*idx);
            out.column_names.remove(*idx);
        }

        Ok(out)
    }

    fn pick(&self, column_1: usize, column_2: usize) -> Signal {
        let mut out = vec![];
        for (x, y) in self.columns[column_1]
            .iter()
            .zip(self.columns[column_2].iter())
        {
            out.push(Record { x: *x, y: *y });
        }
        out
    }
}

pub fn write_csv(path: &Path, data: &[Record]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for r in data {
        writer.serialize(r)?;
    }
    writer.flush()?;
    Ok(())
}

pub type Signal = Vec<Record>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalKind {
    DataFrame,
    Signal,
    Complex,
    Value,
}

/// The data can be a real-valued signal or a complex-valued series
#[derive(Debug, Clone)]
pub enum PipelineIntermediate {
    DataFrame(DataFrame),
    Signal(Signal),
    Complex(Vec<Complex<f64>>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Default)]
pub enum AxisSelection {
    #[default]
    X,
    Y,
}

#[derive(Debug, Clone, Copy, EnumString, Display, Deserialize, Serialize)]
pub enum StepConfig {
    Average,
    Variance,
    SmoothSignal {
        window: usize,
    },
    SmoothReals {
        window: usize,
    },
    AbsoluteValueOfReals,
    FourierTransform,
    InverseFourierTransform,
    PostFFTFormatting,
    SkipFirstEntry,
    SkipFirstComplexEntry,
    Normalize,
    BandpassFilter {
        middle: f64,
        half_width: f64,
    },
    // Assumes x is a time axis in seconds, y is a voltage i axis in Volts
    CurrentCalculator {
        capacitance: f64,
        x1: Option<Record>,
        x2: Option<Record>,
    },
    PickColumns {
        column_1: usize,
        column_2: usize,
    },
    ScaleAxis {
        axis: AxisSelection,
        factor: f64,
    },
    LogAxis {
        axis: AxisSelection,
        base: f64,
    },
}

impl StepConfig {
    pub fn all() -> Vec<Self> {
        vec![
            StepConfig::Average,
            StepConfig::Variance,
            StepConfig::SmoothSignal { window: 100 },
            StepConfig::SmoothReals { window: 100 },
            StepConfig::AbsoluteValueOfReals,
            StepConfig::FourierTransform,
            StepConfig::InverseFourierTransform,
            StepConfig::PostFFTFormatting,
            StepConfig::SkipFirstEntry,
            StepConfig::SkipFirstComplexEntry,
            StepConfig::Normalize,
            StepConfig::BandpassFilter {
                middle: 0.0,
                half_width: 0.0,
            },
            StepConfig::CurrentCalculator {
                capacitance: 0.0,
                x1: None,
                x2: None,
            },
            StepConfig::PickColumns {
                column_1: 0,
                column_2: 1,
            },
            StepConfig::ScaleAxis {
                axis: AxisSelection::X,
                factor: 1.0,
            },
            StepConfig::LogAxis {
                axis: AxisSelection::X,
                base: 10.0,
            },
        ]
    }

    pub fn input_kind(&self) -> SignalKind {
        match self {
            StepConfig::Average => SignalKind::Signal,
            StepConfig::Variance => SignalKind::Signal,
            StepConfig::SmoothSignal { window: _ } => SignalKind::Signal,
            StepConfig::SmoothReals { window: _ } => SignalKind::Complex,
            StepConfig::AbsoluteValueOfReals => SignalKind::Complex,
            StepConfig::FourierTransform => SignalKind::Signal,
            StepConfig::InverseFourierTransform => SignalKind::Complex,
            StepConfig::PostFFTFormatting => SignalKind::Complex,
            StepConfig::SkipFirstEntry => SignalKind::Signal,
            StepConfig::SkipFirstComplexEntry => SignalKind::Complex,
            StepConfig::Normalize => SignalKind::Signal,
            StepConfig::BandpassFilter {
                middle: _,
                half_width: _,
            } => SignalKind::Complex,
            StepConfig::CurrentCalculator {
                capacitance: _,
                x1: _,
                x2: _,
            } => SignalKind::Signal,
            StepConfig::PickColumns {
                column_1: _,
                column_2: _,
            } => SignalKind::DataFrame,
            StepConfig::ScaleAxis { axis: _, factor: _ } => SignalKind::Signal,
            StepConfig::LogAxis { axis: _, base: _ } => SignalKind::Signal,
        }
    }

    pub fn output_kind(&self) -> SignalKind {
        match self {
            StepConfig::Average => SignalKind::Signal,
            StepConfig::Variance => SignalKind::Signal,
            StepConfig::SmoothSignal { window: _ } => SignalKind::Signal,
            StepConfig::SmoothReals { window: _ } => SignalKind::Complex,
            StepConfig::AbsoluteValueOfReals => SignalKind::Complex,
            StepConfig::FourierTransform => SignalKind::Complex,
            StepConfig::InverseFourierTransform => SignalKind::Signal,
            StepConfig::PostFFTFormatting => SignalKind::Signal,
            StepConfig::SkipFirstEntry => SignalKind::Signal,
            StepConfig::SkipFirstComplexEntry => SignalKind::Complex,
            StepConfig::Normalize => SignalKind::Signal,
            StepConfig::BandpassFilter {
                middle: _,
                half_width: _,
            } => SignalKind::Complex,
            StepConfig::CurrentCalculator {
                capacitance: _,
                x1: _,
                x2: _,
            } => SignalKind::Value,
            StepConfig::PickColumns {
                column_1: _,
                column_2: _,
            } => SignalKind::Signal,
            StepConfig::ScaleAxis { axis: _, factor: _ } => SignalKind::Signal,
            StepConfig::LogAxis { axis: _, base: _ } => SignalKind::Signal,
        }
    }
}

impl PartialEq for StepConfig {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (StepConfig::Average, StepConfig::Average) => true,
            (StepConfig::Variance, StepConfig::Variance) => true,
            (StepConfig::SmoothSignal { window: _ }, StepConfig::SmoothSignal { window: _ }) => {
                true
            }
            (StepConfig::SmoothReals { window: _ }, StepConfig::SmoothReals { window: _ }) => true,
            (StepConfig::AbsoluteValueOfReals, StepConfig::AbsoluteValueOfReals) => true,
            (StepConfig::FourierTransform, StepConfig::FourierTransform) => true,
            (StepConfig::InverseFourierTransform, StepConfig::InverseFourierTransform) => true,
            (StepConfig::PostFFTFormatting, StepConfig::PostFFTFormatting) => true,
            (StepConfig::SkipFirstEntry, StepConfig::SkipFirstEntry) => true,
            (StepConfig::SkipFirstComplexEntry, StepConfig::SkipFirstComplexEntry) => true,
            (StepConfig::Normalize, StepConfig::Normalize) => true,
            (
                StepConfig::BandpassFilter {
                    middle: _,
                    half_width: _,
                },
                StepConfig::BandpassFilter {
                    middle: _,
                    half_width: _,
                },
            ) => true,
            (
                StepConfig::PickColumns {
                    column_1: _,
                    column_2: _,
                },
                StepConfig::PickColumns {
                    column_1: _,
                    column_2: _,
                },
            ) => true,
            (
                StepConfig::ScaleAxis { axis: _, factor: _ },
                StepConfig::ScaleAxis { axis: _, factor: _ },
            ) => true,
            _ => false,
        }
    }
}
