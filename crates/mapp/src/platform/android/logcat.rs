use std::{ffi::{CStr, CString}, io};

use android_log_sys::LogPriority;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

pub struct LogcatWriter {
    level: LogPriority,
}

pub struct LogcatMakeWriter;

impl<'a> MakeWriter<'a> for LogcatMakeWriter {
    type Writer = LogcatWriter;

    fn make_writer(&'a self) -> Self::Writer {
        LogcatWriter {
            level: LogPriority::INFO,
        }
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        LogcatWriter {
            level: match *meta.level() {
                Level::TRACE => LogPriority::VERBOSE,
                Level::DEBUG => LogPriority::DEBUG,
                Level::INFO => LogPriority::INFO,
                Level::WARN => LogPriority::WARN,
                Level::ERROR => LogPriority::ERROR,
            },
        }
    }
}

impl io::Write for LogcatWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            android_log_sys::__android_log_write(
                self.level as _,
                c"mtool".as_ptr(),
                CString::new(buf)?.as_ptr(),
            )
        };
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub use android_log_sys;
