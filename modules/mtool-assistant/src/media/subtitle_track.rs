use std::{collections::HashMap, fmt::Display, time::Duration};

use mapp::{
    anyhow::{self, Context},
    base64,
    regex::Regex,
    tracing::debug,
};
use rsubs_lib::{VTTLine, SRT, SSA, VTT};
use time::Time;

use crate::media::{TimedCue, TimedCueData};

use super::TimedRawTrack;

#[derive(Debug, Clone)]
pub enum SubtitleTrack {
    VTT(VTT),
    SRT(SRT),
    SSA(SSA),
}

impl Display for SubtitleTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtitleTrack::VTT(vtt) => vtt.fmt(f),
            SubtitleTrack::SRT(srt) => srt.fmt(f),
            SubtitleTrack::SSA(ssa) => ssa.fmt(f),
        }
    }
}

impl SubtitleTrack {
    pub fn from_raw(raw: TimedRawTrack) -> Result<Self, anyhow::Error> {
        let mut vtt = VTT::default();

        let z = Time::from_hms(0, 0, 0)?;
        for TimedCue {
            id,
            data,
            start_time,
            duration,
        } in raw.cues
        {
            vtt.lines.push(VTTLine {
                identifier: id,
                start: z + start_time,
                end: z + start_time + duration,
                settings: HashMap::default(),
                text: match data {
                    TimedCueData::Text(text) => text,
                    TimedCueData::Binary(_) => String::new(),
                },
            });
        }

        debug!("{vtt}");

        Ok(Self::VTT(vtt))
    }

    pub fn from_lrc(text: &String, duration_ms: u64) -> Result<Self, anyhow::Error> {
        debug!("{text}");

        let re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)")?;

        let mut cues = Vec::new();

        let parse_line = |line| {
            re.captures(line)
                .map(|captures| {
                    (
                        {
                            let minutes: u64 = captures[1].parse().unwrap_or(0);
                            let seconds: u64 = captures[2].parse().unwrap_or(0);
                            let mut milliseconds: u64 = captures[3].parse().unwrap_or(0);
                            if captures[3].len() == 2 {
                                milliseconds *= 10;
                            }

                            Duration::from_secs(minutes * 60 + seconds)
                                + Duration::from_millis(milliseconds)
                        },
                        captures[4].trim().to_string(),
                    )
                })
                .context(line.to_string())
        };

        let mut lines = text.lines().enumerate();

        if let Some((_, line)) = lines.next() {
            let (mut start_time, mut text) = parse_line(line)?;
            for (i, line) in lines {
                let (current_time, current_text) = parse_line(line)?;

                cues.push(TimedCue {
                    id: Some(format!("{i}")),
                    data: TimedCueData::Text(text),
                    start_time,
                    duration: current_time - start_time,
                });
                start_time = current_time;
                text = current_text;
            }

            cues.push(TimedCue {
                id: Some(format!("{}", cues.len() + 1)),
                data: TimedCueData::Text(text),
                start_time,
                duration: Duration::from_millis(duration_ms) - start_time,
            });
        }

        Self::from_raw(TimedRawTrack { cues })
    }

    pub fn data_uri(&self) -> String {
        let mime_type = match self {
            SubtitleTrack::VTT(_) => "text/vtt",
            SubtitleTrack::SRT(_) => "application/x-subrip",
            SubtitleTrack::SSA(_) => "text/ssa",
        };

        format!(
            "data:{mime_type};base64,{}",
            base64::encode(self.to_string().as_bytes())
        )
    }
}
