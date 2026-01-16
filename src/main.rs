#![doc = include_str!("../README.md")]

use std::io::{Write, stdout};
use std::num::FpCategory;
use std::process::{abort, exit};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc;
use std::time::Duration;

use clap::Parser;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128Plus;
use rodio::source::{SeekError, noise};
use rodio::{ChannelCount, Sample, SampleRate, Source};
use tracing::{info, warn};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> Result<(), Error> {
    let args = Args::parse();
    if args.license {
        let _: std::io::Result<()> = stdout()
            .lock()
            .write_all(include_str!("../LICENSE.ISC").as_bytes());
        return Ok(());
    }

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env()?;
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .finish()
        .try_init()?;

    let cancelled = AtomicU32::new(0);
    let (tx, rx) = mpsc::sync_channel(1);
    ctrlc::try_set_handler(move || {
        match cancelled.fetch_add(1, SeqCst) {
            0 => {}
            1 => warn!("Trapped ctrl+C twice. Press ctrc+C again to kill the process."),
            2 => exit(0),
            _ => abort(),
        }
        let _: Result<(), mpsc::SendError<()>> = tx.send(());
    })?;

    let amplitude = if let FpCategory::Normal | FpCategory::Subnormal = args.amplitude.classify()
        && (0.01..=100.0).contains(&args.amplitude)
    {
        args.amplitude
    } else {
        return Err(Error::Amplitude(args.amplitude));
    };
    let stream = rodio::OutputStreamBuilder::open_default_stream()?;
    let sample_rate = stream.config().sample_rate();
    let noise = args.noise.to_noise(sample_rate).amplify(amplitude * 0.01);
    stream.mixer().add(noise);

    info!(
        "Now playing {} noise with an amplitude of {amplitude:.2}%.",
        args.noise
    );
    eprintln!("Press ctrl+C to end the process.");

    let _: Result<(), mpsc::RecvError> = rx.recv();
    eprintln!();
    info!("Closing stream and exiting.");
    drop(stream);
    Ok(())
}

/// Output a continuous stream of (almost) silence.
#[derive(Debug, Parser)]
#[command(version, about, long_about = long_about())]
struct Args {
    /// The type of noise to play
    #[arg(default_value_t = NoiseValue::default(), value_enum)]
    noise: NoiseValue,
    /// The output amplitude in percent
    #[arg(default_value_t = 0.1)]
    amplitude: f32,
    /// Print the license text
    #[arg(short = 'L', long)]
    license: bool,
}

nodyn::nodyn! {
    enum Noise {
        White(noise::WhiteUniform<Xoroshiro128Plus>),
        Gaussian(noise::WhiteGaussian<Xoroshiro128Plus>),
        Triangular(noise::WhiteTriangular<Xoroshiro128Plus>),
        Pink(noise::Pink<Xoroshiro128Plus>),
        Blue(noise::Blue<Xoroshiro128Plus>),
        Violet(noise::Violet<Xoroshiro128Plus>),
        Brownian(noise::Brownian<Xoroshiro128Plus>),
        Velvet(noise::Velvet<Xoroshiro128Plus>),
    }

    impl Iterator {
        type Item = Sample;

        fn next(&mut self) -> Option<Self::Item>;
    }

    impl Source {
        fn current_span_len(&self) -> Option<usize>;
        fn channels(&self) -> ChannelCount;
        fn sample_rate(&self) -> SampleRate;
        fn total_duration(&self) -> Option<Duration>;

        #[inline]
        fn try_seek(&mut self, _: Duration) -> Result<(), SeekError> {
            Ok(())
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, clap::ValueEnum)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
enum NoiseValue {
    White,
    Gaussian,
    Triangular,
    Pink,
    Blue,
    Violet,
    #[default]
    Brownian,
    Velvet,
}

impl NoiseValue {
    fn to_noise(self, sample_rate: SampleRate) -> Noise {
        let func: fn(SampleRate, Xoroshiro128Plus) -> Noise = match self {
            Self::White => |s, r| Noise::White(noise::WhiteUniform::new_with_rng(s, r)),
            Self::Gaussian => |s, r| Noise::Gaussian(noise::WhiteGaussian::new_with_rng(s, r)),
            Self::Triangular => {
                |s, r| Noise::Triangular(noise::WhiteTriangular::new_with_rng(s, r))
            }
            Self::Pink => |s, r| Noise::Pink(noise::Pink::new_with_rng(s, r)),
            Self::Blue => |s, r| Noise::Blue(noise::Blue::new_with_rng(s, r)),
            Self::Violet => |s, r| Noise::Violet(noise::Violet::new_with_rng(s, r)),
            Self::Brownian => |s, r| Noise::Brownian(noise::Brownian::new_with_rng(s, r)),
            Self::Velvet => |s, r| Noise::Velvet(noise::Velvet::new_with_rng(s, r)),
        };
        func(sample_rate, Xoroshiro128Plus::from_seed(SEED))
    }
}

#[derive(pretty_error_debug::Debug, thiserror::Error, displaydoc::Display)]
enum Error {
    /// Could not set up tracing filter
    TracingEnv(#[from] tracing_subscriber::filter::FromEnvError),
    /// Could not set up tracing registry
    TracingInit(#[from] tracing_subscriber::util::TryInitError),
    /// Cannot trap ctrl+C
    CtrlC(#[from] ctrlc::Error),
    /// Could not set up audio stream
    Stream(#[from] rodio::StreamError),
    /// Unsupported amplitude {0:?}
    Amplitude(f32),
}

fn long_about() -> &'static str {
    include_str!("../README.md")
        .split_once("\r\n\r\n")
        .unwrap_or_default()
        .1
}

const SEED: [u8; 16] = *b"Enjoy t. silence";
