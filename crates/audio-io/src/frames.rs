#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StereoFrame {
    pub reference: f32,
    pub measurement: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InputFrame {
    pub frame_index: u64,
    pub loopback_reference: f32,
    pub measurement: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeneratorSample {
    pub frame_index: u64,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadReport {
    pub frames_read: usize,
    pub discontinuity: bool,
    pub finished: bool,
}

impl ReadReport {
    pub fn frames(frames_read: usize) -> Self {
        Self {
            frames_read,
            ..Self::default()
        }
    }
}

pub trait FrameSource: Send {
    fn sample_rate_hz(&self) -> u32;

    fn read_frames(&mut self, destination: &mut Vec<StereoFrame>, max_frames: usize) -> ReadReport;
}
