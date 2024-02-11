use std::{path::PathBuf, time::Duration};

use derivative::Derivative;
use parse_display::ParseError;

use super::{
    error::SrtFileError,
    frame::{SrtDebugFrameData, SrtFrame},
    SrtFrameData,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SrtFile {
    pub file_path: PathBuf,
    pub has_distance: bool,
    pub has_debug: bool,
    pub duration: Duration,
    #[derivative(Debug = "ignore")]
    pub frames: Vec<SrtFrame>,
}

impl SrtFile {
    #[tracing::instrument(ret, err)]
    pub fn open(path: PathBuf) -> Result<Self, SrtFileError> {
        let mut has_distance = false;
        let mut has_debug = false;
        let srt_frames = srtparse::from_file(&path)?
            .iter()
            .map(|i| -> Result<SrtFrame, SrtFileError> {
                let debug_data: Result<SrtDebugFrameData, ParseError> = i.text.parse();
                let mut dd: Option<SrtDebugFrameData> = None;
                if debug_data.is_ok() {
                    dd = Some(debug_data?);
                    has_debug = true;
                }

                let data: Result<SrtFrameData, ParseError> = i.text.parse();
                let mut d: Option<SrtFrameData> = None;
                if data.is_ok() {
                    let ad = data?;
                    has_distance |= ad.distance > 0;
                    d = Some(ad);
                }
                Ok(SrtFrame {
                    start_time_secs: i.start_time.into_duration().as_secs_f32(),
                    end_time_secs: i.end_time.into_duration().as_secs_f32(),
                    data: d,
                    debug_data: dd,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let duration = Duration::from_secs_f32(srt_frames.last().unwrap().end_time_secs);

        Ok(Self {
            file_path: path,
            has_distance,
            has_debug,
            duration,
            frames: srt_frames,
        })
    }
}
