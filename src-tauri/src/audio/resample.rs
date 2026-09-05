//! Downmix + sample-rate conversion to Whisper's 16 kHz mono input.
//!
//! Capture devices hand us whatever they like — 44.1 kHz stereo from a
//! headset, 48 kHz stereo from ScreenCaptureKit — and every source in a
//! session may differ. This converts all of them to one canonical format.
//!
//! The conversion is a windowed-sinc interpolation that does anti-aliasing and
//! rate conversion in a single pass. Naive decimation (taking every third
//! sample of 48 kHz audio) folds everything above 8 kHz back down into the
//! speech band as aliasing noise, which measurably degrades transcription.
//!
//! State is carried across calls, so consecutive blocks from a live stream
//! join seamlessly instead of clicking at each boundary.

use super::types::TARGET_SAMPLE_RATE;

/// Half-width of the interpolation kernel, in input samples. Sixteen taps is
/// the usual quality/cost knee for speech; the whole filter costs well under
/// 1% of a core at 48 kHz.
const HALF_TAPS: isize = 16;

/// Cutoff safety margin. Filtering at exactly Nyquist would leave no
/// transition band, so we pull the cutoff down slightly.
const CUTOFF_SCALE: f32 = 0.92;

pub struct Resampler {
    input_rate: u32,
    channels: usize,
    /// Ratio of input samples consumed per output sample.
    step: f64,
    /// Kernel cutoff as a fraction of the input sample rate.
    cutoff: f32,
    /// Mono input awaiting conversion. The first `HALF_TAPS` samples are
    /// history retained from the previous call so the kernel has a full window.
    pending: Vec<f32>,
    /// Fractional read position within `pending`.
    position: f64,
    primed: bool,
}

impl Resampler {
    pub fn new(input_rate: u32, channels: u16) -> Self {
        let input_rate = input_rate.max(1);
        let step = f64::from(input_rate) / f64::from(TARGET_SAMPLE_RATE);

        // When upsampling there is nothing to alias, so the kernel only needs
        // to band-limit to the *output* Nyquist.
        let cutoff = if step > 1.0 {
            CUTOFF_SCALE / step as f32
        } else {
            CUTOFF_SCALE
        } * 0.5;

        Self {
            input_rate,
            channels: usize::from(channels).max(1),
            step,
            cutoff,
            pending: Vec::new(),
            position: 0.0,
            primed: false,
        }
    }

    /// True when this resampler was built for the given input format. Devices
    /// can change format mid-session, so callers check before reusing one.
    pub fn matches(&self, input_rate: u32, channels: u16) -> bool {
        self.input_rate == input_rate && self.channels == usize::from(channels).max(1)
    }

    /// Converts one block of interleaved input into 16 kHz mono.
    pub fn process(&mut self, interleaved: &[f32]) -> Vec<f32> {
        self.append_mono(interleaved);

        // Prepend silence once at the start of a stream so the very first real
        // sample sits at the centre of the kernel rather than at its edge.
        if !self.primed {
            self.pending.splice(0..0, std::iter::repeat_n(0.0, HALF_TAPS as usize));
            self.position = HALF_TAPS as f64;
            self.primed = true;
        }

        let mut out = Vec::new();
        let limit = self.pending.len() as f64 - HALF_TAPS as f64;

        while self.position < limit {
            out.push(self.interpolate(self.position));
            self.position += self.step;
        }

        self.discard_consumed();
        out
    }

    /// Flushes the tail of the stream, padding with silence so the final real
    /// samples are not lost inside the kernel's lookahead.
    pub fn flush(&mut self) -> Vec<f32> {
        if !self.primed {
            return Vec::new();
        }

        self.pending
            .extend(std::iter::repeat_n(0.0, HALF_TAPS as usize));

        let mut out = Vec::new();
        let limit = self.pending.len() as f64 - HALF_TAPS as f64;
        while self.position < limit {
            out.push(self.interpolate(self.position));
            self.position += self.step;
        }

        self.pending.clear();
        self.position = 0.0;
        self.primed = false;
        out
    }

    /// Averages the interleaved channels down to mono.
    ///
    /// Averaging rather than picking channel 0 matters for meeting audio:
    /// conferencing apps often pan a participant hard to one side, and taking
    /// a single channel would drop them entirely.
    fn append_mono(&mut self, interleaved: &[f32]) {
        if self.channels == 1 {
            self.pending.extend_from_slice(interleaved);
            return;
        }

        let scale = 1.0 / self.channels as f32;
        self.pending.extend(
            interleaved
                .chunks_exact(self.channels)
                .map(|frame| frame.iter().sum::<f32>() * scale),
        );
    }

    /// Windowed-sinc interpolation at a fractional input position.
    fn interpolate(&self, position: f64) -> f32 {
        let centre = position.floor() as isize;
        let frac = (position - position.floor()) as f32;

        let mut acc = 0.0;
        let mut weight_sum = 0.0;

        for tap in (-HALF_TAPS + 1)..=HALF_TAPS {
            let index = centre + tap;
            if index < 0 || index as usize >= self.pending.len() {
                continue;
            }

            // Distance from the (fractional) sampling point to this input
            // sample, in input-sample units.
            let distance = tap as f32 - frac;
            let weight = sinc(2.0 * self.cutoff * distance) * blackman(distance);

            acc += self.pending[index as usize] * weight;
            weight_sum += weight;
        }

        // Normalising by the realised weight sum keeps the gain flat even
        // where the window is clipped by the edges of the buffer.
        if weight_sum.abs() > f32::EPSILON {
            acc / weight_sum
        } else {
            0.0
        }
    }

    /// Drops input we have already read past, keeping one kernel half-width of
    /// history so the next call can filter across the seam.
    fn discard_consumed(&mut self) {
        let keep_from = (self.position.floor() as isize - HALF_TAPS).max(0) as usize;
        if keep_from == 0 {
            return;
        }

        self.pending.drain(..keep_from);
        self.position -= keep_from as f64;
    }
}

fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-6 {
        1.0
    } else {
        let pi_x = std::f32::consts::PI * x;
        pi_x.sin() / pi_x
    }
}

/// Blackman window over the kernel's support, which suppresses the ringing a
/// bare truncated sinc would produce.
fn blackman(distance: f32) -> f32 {
    let half = HALF_TAPS as f32;
    if distance.abs() >= half {
        return 0.0;
    }

    let t = (distance + half) / (2.0 * half);
    let two_pi_t = 2.0 * std::f32::consts::PI * t;
    0.42 - 0.5 * two_pi_t.cos() + 0.08 * (2.0 * two_pi_t).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 48 kHz stream should come out at a third the length.
    #[test]
    fn downsamples_48k_to_16k() {
        let mut resampler = Resampler::new(48_000, 1);
        let input: Vec<f32> = (0..4_800).map(|i| (i as f32 * 0.01).sin()).collect();

        let out = resampler.process(&input);

        // 4800 input samples at 3:1 is 1600 out, modulo kernel lookahead.
        assert!(
            (1_560..=1_600).contains(&out.len()),
            "unexpected output length {}",
            out.len()
        );
    }

    /// Stereo must be averaged, not truncated to the left channel.
    #[test]
    fn averages_stereo_channels() {
        let mut resampler = Resampler::new(16_000, 2);
        // Left silent, right at full scale: mono should land near 0.5.
        let input: Vec<f32> = std::iter::repeat_n([0.0, 1.0], 4_000).flatten().collect();

        let out = resampler.process(&input);
        let settled = &out[out.len() / 2..];
        let mean = settled.iter().sum::<f32>() / settled.len() as f32;

        assert!((mean - 0.5).abs() < 0.01, "expected ~0.5, got {mean}");
    }

    /// Matching input and output rates must not distort amplitude.
    #[test]
    fn passthrough_preserves_amplitude() {
        let mut resampler = Resampler::new(16_000, 1);
        let input: Vec<f32> = (0..1_600)
            .map(|i| (i as f32 / 16_000.0 * 440.0 * std::f32::consts::TAU).sin())
            .collect();

        let out = resampler.process(&input);
        let peak = out.iter().fold(0.0f32, |a, s| a.max(s.abs()));

        assert!((peak - 1.0).abs() < 0.05, "expected ~1.0 peak, got {peak}");
    }
}
