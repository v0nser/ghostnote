//! Linux has no portable system-audio capture story: PulseAudio monitors,
//! PipeWire and JACK all differ, and none is guaranteed present. Rather than
//! guess, we report the gap and let capture run microphone-only.

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::source::{FrameSink, SourceHandle};

pub fn spawn(_sink: FrameSink) -> AudioResult<SourceHandle> {
    Err(AudioError::SystemAudioUnsupported)
}
