use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use chrono::{Datelike, NaiveDateTime, Timelike};

use crate::config::AdaptivePolling;

const HOUR_OF_WEEK_BUCKETS: usize = 7 * 24;

#[derive(Clone)]
pub struct UploaderProfile {
    pub median_upload_interval: Option<f64>,
    pub p75_upload_interval: Option<f64>,
    pub p90_upload_interval: Option<f64>,
    pub intervals_count: usize,
    pub last_upload_timestamp: Option<NaiveDateTime>,
    hour_of_week_score: [f64; HOUR_OF_WEEK_BUCKETS],
}

impl UploaderProfile {
    pub fn build(upload_timestamps: &[NaiveDateTime], cfg: &AdaptivePolling) -> Self {
        let mut upload_times = upload_timestamps.to_vec();
        upload_times.sort_unstable();
        let mut intervals = Vec::with_capacity(upload_times.len().saturating_sub(1));
        for pair in upload_times.windows(2) {
            let delta = (pair[1] - pair[0]).num_minutes();
            if delta > 0 {
                intervals.push(delta as f64);
            }
        }
        let median = percentile(&intervals, 0.5);
        let p75 = percentile(&intervals, 0.75);
        let p90 = percentile(&intervals, 0.9);
        let hour_of_week_score = build_hour_of_week_score(&upload_times, cfg);

        Self {
            median_upload_interval: median,
            p75_upload_interval: p75,
            p90_upload_interval: p90,
            intervals_count: intervals.len(),
            last_upload_timestamp: upload_times.last().copied(),
            hour_of_week_score,
        }
    }

    pub fn time_window_score(&self, now: NaiveDateTime) -> f64 {
        let bucket = hour_of_week_bucket(now);
        self.hour_of_week_score[bucket]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PollAction {
    Poll,
    Skip,
}

#[derive(Debug, Clone, Copy)]
pub enum ForcedPollReason {
    NeverChecked,
    MaxCheckAge,
    BeyondP90,
    InsufficientHistory,
}

impl ForcedPollReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ForcedPollReason::NeverChecked => "never_checked",
            ForcedPollReason::MaxCheckAge => "max_check_age",
            ForcedPollReason::BeyondP90 => "elapsed_beyond_p90",
            ForcedPollReason::InsufficientHistory => "insufficient_history",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PollDecision {
    pub action: PollAction,
    pub final_score: f64,
    pub interval_score: f64,
    pub time_window_score: f64,
    pub elapsed_since_last_upload_minutes: Option<i64>,
    pub elapsed_since_last_check_minutes: Option<i64>,
    pub forced_reason: Option<ForcedPollReason>,
}

pub fn decide_poll(
    now: NaiveDateTime,
    last_check_timestamp: Option<NaiveDateTime>,
    profile: &UploaderProfile,
    cfg: &AdaptivePolling,
) -> PollDecision {
    let elapsed_since_last_upload_minutes = profile
        .last_upload_timestamp
        .map(|last_upload| (now - last_upload).num_minutes().max(0));
    let elapsed_since_last_check_minutes =
        last_check_timestamp.map(|last_check| (now - last_check).num_minutes().max(0));

    let forced_reason = if last_check_timestamp.is_none() {
        Some(ForcedPollReason::NeverChecked)
    } else if profile.intervals_count < cfg.min_history_intervals {
        Some(ForcedPollReason::InsufficientHistory)
    } else if elapsed_since_last_check_minutes.is_some_and(|elapsed| elapsed >= cfg.force_check_max_age_minutes) {
        Some(ForcedPollReason::MaxCheckAge)
    } else if should_force_beyond_p90(
        elapsed_since_last_upload_minutes,
        elapsed_since_last_check_minutes,
        profile.p90_upload_interval,
        cfg.forced_beyond_p90_cooldown_minutes,
    ) {
        Some(ForcedPollReason::BeyondP90)
    } else {
        None
    };

    let interval_score = interval_score(elapsed_since_last_upload_minutes, profile);
    let time_window_score = profile.time_window_score(now);
    let final_score = combine_score(
        interval_score,
        time_window_score,
        elapsed_since_last_upload_minutes,
        cfg,
    );

    let action = if forced_reason.is_some() || final_score >= cfg.threshold {
        PollAction::Poll
    } else {
        PollAction::Skip
    };

    PollDecision {
        action,
        final_score,
        interval_score,
        time_window_score,
        elapsed_since_last_upload_minutes,
        elapsed_since_last_check_minutes,
        forced_reason,
    }
}

fn combine_score(
    interval_score: f64,
    time_window_score: f64,
    elapsed_since_last_upload_minutes: Option<i64>,
    cfg: &AdaptivePolling,
) -> f64 {
    let weight_sum = (cfg.interval_weight + cfg.time_window_weight).max(f64::EPSILON);
    let mut score = (cfg.interval_weight * interval_score + cfg.time_window_weight * time_window_score) / weight_sum;
    if elapsed_since_last_upload_minutes.is_some_and(|elapsed| elapsed <= cfg.burst_window_minutes) {
        score += cfg.burst_score_boost;
    }
    score.clamp(0.0, 1.0)
}

fn should_force_beyond_p90(
    elapsed_since_last_upload_minutes: Option<i64>,
    elapsed_since_last_check_minutes: Option<i64>,
    p90_upload_interval: Option<f64>,
    cooldown_minutes: i64,
) -> bool {
    let Some((elapsed_upload, p90)) = elapsed_since_last_upload_minutes.zip(p90_upload_interval) else {
        return false;
    };
    if (elapsed_upload as f64) < p90 {
        return false;
    }
    if cooldown_minutes <= 0 {
        return true;
    }
    elapsed_since_last_check_minutes.is_none_or(|elapsed_check| elapsed_check >= cooldown_minutes)
}

fn interval_score(elapsed_since_last_upload_minutes: Option<i64>, profile: &UploaderProfile) -> f64 {
    let Some(elapsed) = elapsed_since_last_upload_minutes else {
        return 0.0;
    };
    let elapsed = elapsed as f64;
    let Some(median) = profile.median_upload_interval else {
        return 0.0;
    };
    let p75 = profile.p75_upload_interval.unwrap_or(median * 1.3);
    let p90 = profile.p90_upload_interval.unwrap_or(median * 1.8);

    if elapsed <= median * 0.5 {
        return 0.05;
    }
    if elapsed <= median {
        return lerp(0.05, 0.5, normalize(elapsed, median * 0.5, median));
    }
    if elapsed <= p75 {
        return lerp(0.5, 0.75, normalize(elapsed, median, p75));
    }
    if elapsed <= p90 {
        return lerp(0.75, 0.95, normalize(elapsed, p75, p90));
    }
    1.0
}

fn build_hour_of_week_score(upload_timestamps: &[NaiveDateTime], cfg: &AdaptivePolling) -> [f64; HOUR_OF_WEEK_BUCKETS] {
    let mut histogram = [0.0_f64; HOUR_OF_WEEK_BUCKETS];
    for ts in upload_timestamps {
        histogram[hour_of_week_bucket(*ts)] += 1.0;
    }
    let mut smoothed = [0.0_f64; HOUR_OF_WEEK_BUCKETS];
    let span = usize::from(cfg.histogram_neighbor_hours);
    for idx in 0..HOUR_OF_WEEK_BUCKETS {
        let mut sum = histogram[idx];
        for step in 1..=span {
            let weight = cfg.histogram_neighbor_decay.powi(step as i32);
            let prev = (idx + HOUR_OF_WEEK_BUCKETS - step) % HOUR_OF_WEEK_BUCKETS;
            let next = (idx + step) % HOUR_OF_WEEK_BUCKETS;
            sum += (histogram[prev] + histogram[next]) * weight;
        }
        smoothed[idx] = sum;
    }
    let max_score = smoothed.iter().copied().fold(0.0_f64, f64::max);
    if max_score > 0.0 {
        smoothed.iter_mut().for_each(|v| *v /= max_score);
    }
    smoothed
}

fn hour_of_week_bucket(ts: NaiveDateTime) -> usize {
    ts.weekday().num_days_from_monday() as usize * 24 + ts.hour() as usize
}

fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let index = ((sorted.len() as f64 * p).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    Some(sorted[index])
}

fn normalize(value: f64, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return 1.0;
    }
    ((value - lo) / (hi - lo)).clamp(0.0, 1.0)
}

fn lerp(lo: f64, hi: f64, t: f64) -> f64 {
    lo + (hi - lo) * t
}

#[derive(Default, Clone, Copy)]
struct UploaderMetric {
    poll_attempts: u64,
    skipped_checks: u64,
    forced_polls: u64,
    uploads_found: u64,
    check_gap_minutes_total: u64,
    check_gap_samples: u64,
}

#[derive(Default)]
struct AdaptiveMetrics {
    total_poll_attempts: u64,
    total_skipped_checks: u64,
    total_forced_polls: u64,
    total_uploads_found: u64,
    by_uploader: HashMap<i32, UploaderMetric>,
}

static ADAPTIVE_METRICS: LazyLock<Mutex<AdaptiveMetrics>> = LazyLock::new(|| Mutex::new(AdaptiveMetrics::default()));

pub fn record_decision(uploader_id: i32, decision: &PollDecision) {
    let mut metrics = ADAPTIVE_METRICS.lock().expect("adaptive metrics lock poisoned");
    match decision.action {
        PollAction::Poll => metrics.total_poll_attempts += 1,
        PollAction::Skip => metrics.total_skipped_checks += 1,
    }
    if decision.forced_reason.is_some() {
        metrics.total_forced_polls += 1;
    }
    let uploader_metrics = metrics.by_uploader.entry(uploader_id).or_default();
    match decision.action {
        PollAction::Poll => uploader_metrics.poll_attempts += 1,
        PollAction::Skip => uploader_metrics.skipped_checks += 1,
    }
    if decision.forced_reason.is_some() {
        uploader_metrics.forced_polls += 1;
    }
    if let Some(gap) = decision.elapsed_since_last_check_minutes {
        uploader_metrics.check_gap_minutes_total += gap.max(0) as u64;
        uploader_metrics.check_gap_samples += 1;
    }
}

pub fn record_uploads_found(uploader_id: i32, uploads: usize) {
    if uploads == 0 {
        return;
    }
    let mut metrics = ADAPTIVE_METRICS.lock().expect("adaptive metrics lock poisoned");
    metrics.total_uploads_found += uploads as u64;
    let uploader_metrics = metrics.by_uploader.entry(uploader_id).or_default();
    uploader_metrics.uploads_found += uploads as u64;
}

pub fn log_metrics_snapshot() {
    let metrics = ADAPTIVE_METRICS.lock().expect("adaptive metrics lock poisoned");
    info!(
        "adaptive polling metrics: poll_attempts={}, skipped_checks={}, forced_polls={}, uploads_found={}, uploader_count={}",
        metrics.total_poll_attempts,
        metrics.total_skipped_checks,
        metrics.total_forced_polls,
        metrics.total_uploads_found,
        metrics.by_uploader.len()
    );
}
