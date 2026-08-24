use crate::{
    common::{ArchUnitError, CheckLogger},
    metrics::MetricMeasurement,
};

pub(super) fn log_measurements(
    logger: &CheckLogger<'_>,
    measurements: &[MetricMeasurement],
    threshold: Option<f64>,
) -> Result<(), ArchUnitError> {
    for measurement in measurements {
        logger.log_metric(
            measurement.metric_name(),
            measurement.identifier(),
            measurement.value(),
            threshold,
        )?;
    }
    Ok(())
}
