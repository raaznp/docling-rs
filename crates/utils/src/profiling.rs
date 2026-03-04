use docling_core::ConversionResult;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilingScope {
    Document,
    Page,
    Model,
}

/// A simple RAII time recorder that records elapsed time into `ConversionResult.timings`.
pub struct TimeRecorder<'a> {
    conv_res: &'a mut ConversionResult,
    label: String,
    start: Instant,
}

impl<'a> TimeRecorder<'a> {
    pub fn new(
        conv_res: &'a mut ConversionResult,
        label: impl Into<String>,
        _scope: ProfilingScope,
    ) -> Self {
        Self {
            conv_res,
            label: label.into(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for TimeRecorder<'a> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        self.conv_res.timings.record(&self.label, elapsed);
    }
}
