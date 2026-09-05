//! Splits a continuous 16 kHz mono stream into utterances.
//!
//! Whisper is a batch model: it wants a bounded clip, and it is most accurate
//! when that clip contains a whole thought rather than a fragment cut
//! mid-word. So instead of transcribing fixed 10-second windows, we cut on
//! silence — which lines up with natural speech boundaries and, as a bonus,
//! means we never send Whisper an empty room.
//!
//! Detection is energy-based with an adaptive noise floor. A fixed threshold
//! fails badly in practice: a laptop fan, an air conditioner or a noisy line
//! sits at a level that would either swallow quiet speech or trigger
//! constantly, and it differs per source and per room.

use super::types::{Speaker, Utterance, TARGET_SAMPLE_RATE};
use super::vad;

/// Analysis frame. Matches [`vad`] — 20 ms, WebRTC-style.
const FRAME_MS: usize = vad::FRAME_MS;
const FRAME_SAMPLES: usize = vad::FRAME_SAMPLES;

/// Trailing silence that ends an utterance for the user's mic (echo gating).
const SILENCE_TO_CUT_MS: usize = 250;

/// End-of-question pause. See [`vad::INTERVIEWER_HANGOVER_MS`].
const QUESTION_SILENCE_MS: usize = vad::INTERVIEWER_HANGOVER_MS;

/// Audio kept from *before* speech was detected. The detector inevitably fires
/// a frame or two late, and without this the leading consonant gets clipped —
/// "sending" becomes "ending".
const PREROLL_MS: usize = 300;

/// Hard cap on utterance length. Whisper degrades past its 30 s context
/// window, so we cut before reaching it even if the speaker has not paused.
const MAX_UTTERANCE_MS: usize = 25_000;

/// Utterances shorter than this are almost always a cough, a keystroke or a
/// door closing. Transcribing them wastes a sidecar invocation and produces
/// hallucinated text.
const MIN_SPEECH_MS: usize = 320;

/// Absolute floor. Below this an input is silent regardless of what the
/// adaptive estimate says, which stops the detector chasing its own tail on a
/// perfectly clean digital-silence stream.
const ABSOLUTE_SILENCE_RMS: f32 = vad::ABSOLUTE_SILENCE_RMS;

/// How much recent audio the noise floor is estimated from. Long enough to
/// span the gaps between words — a window containing no quiet frame reads the
/// quietest part of a word as background — and short enough to follow a room
/// that gets noisier partway through a meeting.
const NOISE_WINDOW_MS: usize = 3_000;
const NOISE_WINDOW_FRAMES: usize = NOISE_WINDOW_MS / FRAME_MS;

const fn ms_to_samples(ms: usize) -> usize {
    (TARGET_SAMPLE_RATE as usize / 1000) * ms
}

pub struct Segmenter {
    speaker: Speaker,

    /// Unprocessed tail, shorter than one frame.
    partial: Vec<f32>,
    /// Audio for the utterance currently being built.
    current: Vec<f32>,
    /// Rolling pre-speech history.
    preroll: std::collections::VecDeque<f32>,

    in_speech: bool,
    /// Consecutive silent samples seen since the last speech frame.
    trailing_silence: usize,
    /// Samples in the current utterance that were classified as speech.
    /// Counted separately from the clip length, which also contains pre-roll
    /// and a trailing tail — a clip can be half a second long while holding
    /// only a 60 ms door slam.
    speech_samples: usize,
    /// RMS of each frame in the recent past, oldest first.
    recent_rms: std::collections::VecDeque<f32>,
    /// Background estimate: the quietest frame in `recent_rms`.
    noise_floor: f32,

    /// Total samples consumed, for timestamps.
    samples_seen: u64,
    /// Sample index where the current utterance began.
    utterance_start: u64,
    /// How much of `current` was last handed out as a live preview.
    last_preview_len: usize,
}

impl Segmenter {
    pub fn new(speaker: Speaker) -> Self {
        Self {
            speaker,
            partial: Vec::with_capacity(FRAME_SAMPLES),
            current: Vec::new(),
            preroll: std::collections::VecDeque::with_capacity(ms_to_samples(PREROLL_MS)),
            in_speech: false,
            trailing_silence: 0,
            speech_samples: 0,
            recent_rms: std::collections::VecDeque::with_capacity(NOISE_WINDOW_FRAMES),
            noise_floor: ABSOLUTE_SILENCE_RMS,
            samples_seen: 0,
            utterance_start: 0,
            last_preview_len: 0,
        }
    }

    /// Feeds 16 kHz mono audio in, and gets completed utterances out.
    pub fn push(&mut self, samples: &[f32]) -> Vec<Utterance> {
        let mut done = Vec::new();

        self.partial.extend_from_slice(samples);

        while self.partial.len() >= FRAME_SAMPLES {
            let frame: Vec<f32> = self.partial.drain(..FRAME_SAMPLES).collect();
            if let Some(utterance) = self.consume_frame(&frame) {
                done.push(utterance);
            }
        }

        done
    }

    /// Ends the stream, emitting whatever speech is still buffered.
    pub fn flush(&mut self) -> Option<Utterance> {
        if !self.partial.is_empty() {
            let tail = std::mem::take(&mut self.partial);
            self.current.extend_from_slice(&tail);
            self.samples_seen += tail.len() as u64;
        }

        self.in_speech.then(|| self.finish_utterance())?
    }

    /// Snapshot of the question currently being asked. Whisper runs on these
    /// so the model is already loaded when they pause. The coach does not
    /// answer a preview — that would be a guess.
    pub fn take_preview(&mut self) -> Option<Utterance> {
        if self.speaker != Speaker::Participant || !self.in_speech {
            return None;
        }
        if self.trailing_silence > 0 {
            return None;
        }

        let speech_ms = self.speech_samples * 1000 / TARGET_SAMPLE_RATE as usize;
        if speech_ms < 1_400 {
            return None;
        }

        let grown = self.current.len().saturating_sub(self.last_preview_len);
        if grown < ms_to_samples(1_600) {
            return None;
        }

        self.last_preview_len = self.current.len();
        let clip_ms = self.current.len() * 1000 / TARGET_SAMPLE_RATE as usize;
        let start_ms = self.utterance_start * 1000 / u64::from(TARGET_SAMPLE_RATE);

        Some(Utterance {
            speaker: self.speaker,
            samples: self.current.clone(),
            start_ms,
            end_ms: start_ms + clip_ms as u64,
            preview: true,
        })
    }

    fn consume_frame(&mut self, frame: &[f32]) -> Option<Utterance> {
        let rms = vad::rms(frame);
        let speech = self.classify(rms);

        self.samples_seen += frame.len() as u64;

        if speech {
            if !self.in_speech {
                self.begin_utterance();
            }
            self.current.extend_from_slice(frame);
            self.trailing_silence = 0;
            self.speech_samples += frame.len();
        } else if self.in_speech {
            // Keep trailing silence in the clip for now: Whisper transcribes
            // more accurately with a little room tone after the final word,
            // and we trim the excess when the utterance closes.
            self.current.extend_from_slice(frame);
            self.trailing_silence += frame.len();
        } else {
            self.remember_preroll(frame);
        }

        let ended =
            self.in_speech && self.trailing_silence >= self.silence_to_cut_samples();
        let too_long = self.in_speech && self.current.len() >= ms_to_samples(MAX_UTTERANCE_MS);

        if ended || too_long {
            return self.finish_utterance();
        }

        None
    }

    fn silence_to_cut_samples(&self) -> usize {
        match self.speaker {
            Speaker::You => ms_to_samples(SILENCE_TO_CUT_MS),
            Speaker::Participant => vad::INTERVIEWER_HANGOVER_SAMPLES,
        }
    }

    /// Decides whether a frame is speech, and folds the frame into the
    /// noise-floor estimate.
    ///
    /// The floor is the quietest frame in the last [`NOISE_WINDOW_MS`], which
    /// matters mostly for what it does *not* depend on: the speech decision.
    /// An estimate updated only on frames it had already called silence
    /// latches. The floor starts at digital silence, every real room sits
    /// above the threshold that implies, so frame one reads as speech, the
    /// floor never updates, and the detector never calls silence again for
    /// the rest of the meeting.
    fn classify(&mut self, rms: f32) -> bool {
        self.recent_rms.push_back(rms);
        if self.recent_rms.len() > NOISE_WINDOW_FRAMES {
            self.recent_rms.pop_front();
        }

        self.noise_floor = self
            .recent_rms
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
            .max(ABSOLUTE_SILENCE_RMS);

        vad::is_speech(rms, self.noise_floor)
    }

    fn begin_utterance(&mut self) {
        self.in_speech = true;
        self.trailing_silence = 0;
        self.speech_samples = 0;

        self.current.clear();
        self.current.extend(self.preroll.iter().copied());

        // The utterance starts where the pre-roll starts, not where detection
        // fired, so timestamps line up with what the audio actually contains.
        self.utterance_start = self.samples_seen.saturating_sub(self.preroll.len() as u64);
        self.preroll.clear();
        self.last_preview_len = 0;
    }

    fn remember_preroll(&mut self, frame: &[f32]) {
        let capacity = ms_to_samples(PREROLL_MS);
        self.preroll.extend(frame.iter().copied());
        while self.preroll.len() > capacity {
            self.preroll.pop_front();
        }
    }

    fn finish_utterance(&mut self) -> Option<Utterance> {
        self.in_speech = false;

        let mut samples = std::mem::take(&mut self.current);
        let silence = self.trailing_silence;
        let speech_samples = self.speech_samples;
        self.trailing_silence = 0;
        self.speech_samples = 0;
        self.last_preview_len = 0;

        // Judge on speech content, not clip length: the clip always includes
        // pre-roll and a tail, which together clear the minimum on their own.
        let speech_ms = speech_samples * 1000 / TARGET_SAMPLE_RATE as usize;
        let min_ms = match self.speaker {
            Speaker::You => MIN_SPEECH_MS,
            Speaker::Participant => 450,
        };
        if speech_ms < min_ms {
            return None;
        }

        // Trim the long tail of silence back to a short, natural-sounding one.
        let keep_tail = ms_to_samples(200);
        if silence > keep_tail {
            samples.truncate(samples.len().saturating_sub(silence - keep_tail));
        }

        let clip_ms = samples.len() * 1000 / TARGET_SAMPLE_RATE as usize;
        let start_ms = self.utterance_start * 1000 / u64::from(TARGET_SAMPLE_RATE);

        Some(Utterance {
            speaker: self.speaker,
            end_ms: start_ms + clip_ms as u64,
            start_ms,
            samples,
            preview: false,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn tone(ms: usize, amplitude: f32) -> Vec<f32> {
        (0..ms_to_samples(ms))
            .map(|i| {
                amplitude * (i as f32 / TARGET_SAMPLE_RATE as f32 * 220.0 * std::f32::consts::TAU).sin()
            })
            .collect()
    }

    fn silence(ms: usize) -> Vec<f32> {
        vec![0.0; ms_to_samples(ms)]
    }

    #[test]
    fn emits_utterance_after_trailing_silence() {
        let mut segmenter = Segmenter::new(Speaker::You);

        assert!(segmenter.push(&silence(500)).is_empty());
        assert!(segmenter.push(&tone(1_000, 0.3)).is_empty());

        let out = segmenter.push(&silence(SILENCE_TO_CUT_MS + 100));
        assert_eq!(out.len(), 1, "expected exactly one utterance");
        assert_eq!(out[0].speaker, Speaker::You);
        assert!(out[0].duration_ms() >= 1_000);
    }

    #[test]
    fn ignores_short_transients() {
        let mut segmenter = Segmenter::new(Speaker::Participant);

        segmenter.push(&silence(400));
        segmenter.push(&tone(60, 0.8)); // a keystroke, not speech
        let out = segmenter.push(&silence(QUESTION_SILENCE_MS + 100));

        assert!(out.is_empty(), "short transient should not be emitted");
    }

    #[test]
    fn caps_utterance_length() {
        let mut segmenter = Segmenter::new(Speaker::You);
        segmenter.push(&silence(200));

        let out = segmenter.push(&tone(MAX_UTTERANCE_MS + 2_000, 0.3));

        assert!(!out.is_empty(), "a monologue must still be cut");
        let longest = out.iter().map(Utterance::duration_ms).max().unwrap();
        assert!(
            longest <= MAX_UTTERANCE_MS as u64 + 100,
            "utterance ran to {longest}ms"
        );
    }

    /// A room whose background sits well above `ABSOLUTE_SILENCE_RMS` — a fan,
    /// an air conditioner, a noisy line. This is the common case, not an edge
    /// case, and it used to latch the detector into permanent speech.
    #[test]
    fn steady_background_is_never_speech() {
        let mut segmenter = Segmenter::new(Speaker::You);

        let out = segmenter.push(&tone(MAX_UTTERANCE_MS + 2_000, 0.02));

        assert!(
            out.is_empty(),
            "room tone emitted {} utterance(s)",
            out.len()
        );
        assert!(segmenter.flush().is_none(), "room tone emitted a tail");
    }

    #[test]
    fn detects_speech_over_steady_background() {
        let mut segmenter = Segmenter::new(Speaker::You);

        assert!(segmenter.push(&tone(1_000, 0.02)).is_empty());
        assert!(segmenter.push(&tone(1_200, 0.35)).is_empty());
        let out = segmenter.push(&tone(SILENCE_TO_CUT_MS + 200, 0.02));

        assert_eq!(out.len(), 1, "expected exactly one utterance");
        assert!(
            out[0].duration_ms() < MAX_UTTERANCE_MS as u64,
            "utterance ran to the cap instead of cutting on silence"
        );
    }

    #[test]
    fn preroll_keeps_word_onset() {
        let mut segmenter = Segmenter::new(Speaker::You);
        segmenter.push(&silence(1_000));
        segmenter.push(&tone(800, 0.3));
        let out = segmenter.push(&silence(SILENCE_TO_CUT_MS + 100));

        // The clip should be longer than the speech itself, because pre-roll
        // and a trimmed tail are included.
        assert!(out[0].duration_ms() > 800);
    }
}
