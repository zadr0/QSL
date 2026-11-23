use crate::{model::*, Result};
#[allow(dead_code)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}
fn encode_geo_plain(value: Geo) -> Result<Vec<u8>> {
    let record = generated::GeoPlain::try_new(value.zone, value.city, value.altitude_m)?;
    Ok(record.serialize_checked()?)
}
fn decode_geo_plain(bytes: &[u8]) -> Result<Geo> {
    let record = generated::GeoPlain::deserialize_checked(bytes)?;
    Ok(Geo {
        zone: record.zone(),
        altitude_m: record.altitude_m(),
        city: record.city,
    })
}

fn encode_auth_plain(value: Auth) -> Result<Vec<u8>> {
    let record = generated::AuthPlain::try_new(value.session_id, value.scope, value.quota)?;
    Ok(record.serialize_checked()?)
}
fn decode_auth_plain(bytes: &[u8]) -> Result<Auth> {
    let record = generated::AuthPlain::deserialize_checked(bytes)?;
    Ok(Auth {
        session_id: record.session_id(),
        quota: record.quota(),
        scope: record.scope,
    })
}

fn encode_context_plain(value: Context) -> Result<Vec<u8>> {
    let record = generated::ContextPlain::try_new(
        value.gateway,
        value.label,
        value.geo.map(encode_geo_plain).transpose()?,
        value.auth.map(encode_auth_plain).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_context_plain(bytes: &[u8]) -> Result<Context> {
    let record = generated::ContextPlain::deserialize_checked(bytes)?;
    Ok(Context {
        gateway: record.gateway(),
        label: record.label,
        geo: record.geo.as_deref().map(decode_geo_plain).transpose()?,
        auth: record.auth.as_deref().map(decode_auth_plain).transpose()?,
    })
}

fn encode_stats_plain(value: Stats) -> Result<Vec<u8>> {
    let record = generated::StatsPlain::try_new(value.minimum, value.maximum, value.error_count)?;
    Ok(record.serialize_checked()?)
}
fn decode_stats_plain(bytes: &[u8]) -> Result<Stats> {
    let record = generated::StatsPlain::deserialize_checked(bytes)?;
    Ok(Stats {
        error_count: record.error_count(),
        minimum: record.minimum,
        maximum: record.maximum,
    })
}

fn encode_reading_plain(value: Reading) -> Result<Vec<u8>> {
    let record = generated::ReadingPlain::try_new(
        value.sensor_id,
        value.values,
        value.unit,
        value.scale,
        value.stats.map(encode_stats_plain).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_reading_plain(bytes: &[u8]) -> Result<Reading> {
    let record = generated::ReadingPlain::deserialize_checked(bytes)?;
    Ok(Reading {
        scale: record.scale(),
        sensor_id: record.sensor_id,
        values: record.values,
        unit: record.unit,
        stats: record
            .stats
            .as_deref()
            .map(decode_stats_plain)
            .transpose()?,
    })
}

fn encode_limits_plain(value: Limits) -> Result<Vec<u8>> {
    let record = generated::LimitsPlain::try_new(value.low, value.high)?;
    Ok(record.serialize_checked()?)
}
fn decode_limits_plain(bytes: &[u8]) -> Result<Limits> {
    let record = generated::LimitsPlain::deserialize_checked(bytes)?;
    Ok(Limits {
        low: record.low(),
        high: record.high(),
    })
}

fn encode_patch_plain(value: Patch) -> Result<Vec<u8>> {
    let record = generated::PatchPlain::try_new(
        value.firmware,
        value.enabled.map(u8::from),
        value.limits.map(encode_limits_plain).transpose()?,
        value.reboot_after_ms,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_patch_plain(bytes: &[u8]) -> Result<Patch> {
    let record = generated::PatchPlain::deserialize_checked(bytes)?;
    Ok(Patch {
        enabled: record.enabled().map(|item| item != 0),
        reboot_after_ms: record.reboot_after_ms(),
        firmware: record.firmware,
        limits: record
            .limits
            .as_deref()
            .map(decode_limits_plain)
            .transpose()?,
    })
}

fn encode_update_plain(value: Update) -> Result<Vec<u8>> {
    let record = generated::UpdatePlain::try_new(
        value.entity_id,
        value.mask,
        value.patch.map(encode_patch_plain).transpose()?,
        value.generation,
        value.reason,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_update_plain(bytes: &[u8]) -> Result<Update> {
    let record = generated::UpdatePlain::deserialize_checked(bytes)?;
    Ok(Update {
        generation: record.generation(),
        entity_id: record.entity_id,
        mask: record.mask,
        patch: record
            .patch
            .as_deref()
            .map(decode_patch_plain)
            .transpose()?,
        reason: record.reason,
    })
}

fn encode_retry_plain(value: Retry) -> Result<Vec<u8>> {
    let record = generated::RetryPlain::try_new(value.attempt, value.backoff_ms, value.token)?;
    Ok(record.serialize_checked()?)
}
fn decode_retry_plain(bytes: &[u8]) -> Result<Retry> {
    let record = generated::RetryPlain::deserialize_checked(bytes)?;
    Ok(Retry {
        attempt: record.attempt(),
        backoff_ms: record.backoff_ms(),
        token: record.token,
    })
}

fn encode_notice_plain(value: Notice) -> Result<Vec<u8>> {
    let record = generated::NoticePlain {
        severity: value.severity,
        code: value.code,
        related: value.related,
        summary: value.summary,
        retry: value.retry.map(encode_retry_plain).transpose()?,
    };
    Ok(record.serialize_checked()?)
}
fn decode_notice_plain(bytes: &[u8]) -> Result<Notice> {
    let record = generated::NoticePlain::deserialize_checked(bytes)?;
    Ok(Notice {
        severity: record.severity,
        code: record.code,
        related: record.related,
        summary: record.summary,
        retry: record
            .retry
            .as_deref()
            .map(decode_retry_plain)
            .transpose()?,
    })
}

fn encode_heartbeat_plain(value: Heartbeat) -> Result<Vec<u8>> {
    let record = generated::HeartbeatPlain::try_new(value.sequence, value.uptime_ms, value.lag_ms)?;
    Ok(record.serialize_checked()?)
}
fn decode_heartbeat_plain(bytes: &[u8]) -> Result<Heartbeat> {
    let record = generated::HeartbeatPlain::deserialize_checked(bytes)?;
    Ok(Heartbeat {
        lag_ms: record.lag_ms(),
        sequence: record.sequence,
        uptime_ms: record.uptime_ms,
    })
}

fn encode_event_plain(value: Event) -> Result<Vec<u8>> {
    let record = generated::EventPlain {
        sequence: value.sequence,
        topic: value.topic,
        affected: value.affected,
        note: value.note,
        retry: value.retry.map(encode_retry_plain).transpose()?,
        patch: value.patch.map(encode_patch_plain).transpose()?,
    };
    Ok(record.serialize_checked()?)
}
fn decode_event_plain(bytes: &[u8]) -> Result<Event> {
    let record = generated::EventPlain::deserialize_checked(bytes)?;
    Ok(Event {
        sequence: record.sequence,
        topic: record.topic,
        affected: record.affected,
        note: record.note,
        retry: record
            .retry
            .as_deref()
            .map(decode_retry_plain)
            .transpose()?,
        patch: record
            .patch
            .as_deref()
            .map(decode_patch_plain)
            .transpose()?,
    })
}

fn encode_envelope_plain(value: Envelope) -> Result<Vec<u8>> {
    let (kind, payload) = match value.body.ok_or("missing body")? {
        Body::Reading(child) => (1, encode_reading_plain(child)?),
        Body::Update(child) => (2, encode_update_plain(child)?),
        Body::Notice(child) => (3, encode_notice_plain(child)?),
        Body::Heartbeat(child) => (4, encode_heartbeat_plain(child)?),
        Body::Event(child) => (5, encode_event_plain(child)?),
    };
    let record = generated::EnvelopePlain::try_new(
        value.request_id,
        value.revision,
        value.timestamp_ms,
        kind,
        payload,
        value.deadline_ms,
        value.device_id,
        value.tracing.map(u8::from),
        value.trace,
        value.context.map(encode_context_plain).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_envelope_plain(bytes: &[u8]) -> Result<Envelope> {
    let record = generated::EnvelopePlain::deserialize_checked(bytes)?;
    Ok(Envelope {
        deadline_ms: record.deadline_ms(),
        device_id: record.device_id(),
        tracing: record.tracing().map(|item| item != 0),
        request_id: record.request_id,
        revision: record.revision,
        timestamp_ms: record.timestamp_ms,
        trace: record.trace,
        context: record
            .context
            .as_deref()
            .map(decode_context_plain)
            .transpose()?,
        body: Some(match record.kind {
            1 => Body::Reading(decode_reading_plain(&record.payload)?),
            2 => Body::Update(decode_update_plain(&record.payload)?),
            3 => Body::Notice(decode_notice_plain(&record.payload)?),
            4 => Body::Heartbeat(decode_heartbeat_plain(&record.payload)?),
            5 => Body::Event(decode_event_plain(&record.payload)?),
            _ => return Err("unknown QSL body".into()),
        }),
    })
}

fn encode_geo_bitmap(value: Geo) -> Result<Vec<u8>> {
    let record = generated::GeoBitmap::try_new(value.zone, value.city, value.altitude_m)?;
    Ok(record.serialize_checked()?)
}
fn decode_geo_bitmap(bytes: &[u8]) -> Result<Geo> {
    let record = generated::GeoBitmap::deserialize_checked(bytes)?;
    Ok(Geo {
        zone: record.zone(),
        altitude_m: record.altitude_m(),
        city: record.city,
    })
}

fn encode_auth_bitmap(value: Auth) -> Result<Vec<u8>> {
    let record = generated::AuthBitmap::try_new(value.session_id, value.scope, value.quota)?;
    Ok(record.serialize_checked()?)
}
fn decode_auth_bitmap(bytes: &[u8]) -> Result<Auth> {
    let record = generated::AuthBitmap::deserialize_checked(bytes)?;
    Ok(Auth {
        session_id: record.session_id(),
        quota: record.quota(),
        scope: record.scope,
    })
}

fn encode_context_bitmap(value: Context) -> Result<Vec<u8>> {
    let record = generated::ContextBitmap::try_new(
        value.gateway,
        value.label,
        value.geo.map(encode_geo_bitmap).transpose()?,
        value.auth.map(encode_auth_bitmap).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_context_bitmap(bytes: &[u8]) -> Result<Context> {
    let record = generated::ContextBitmap::deserialize_checked(bytes)?;
    Ok(Context {
        gateway: record.gateway(),
        label: record.label,
        geo: record.geo.as_deref().map(decode_geo_bitmap).transpose()?,
        auth: record.auth.as_deref().map(decode_auth_bitmap).transpose()?,
    })
}

fn encode_stats_bitmap(value: Stats) -> Result<Vec<u8>> {
    let record = generated::StatsBitmap::try_new(value.minimum, value.maximum, value.error_count)?;
    Ok(record.serialize_checked()?)
}
fn decode_stats_bitmap(bytes: &[u8]) -> Result<Stats> {
    let record = generated::StatsBitmap::deserialize_checked(bytes)?;
    Ok(Stats {
        error_count: record.error_count(),
        minimum: record.minimum,
        maximum: record.maximum,
    })
}

fn encode_reading_bitmap(value: Reading) -> Result<Vec<u8>> {
    let record = generated::ReadingBitmap::try_new(
        value.sensor_id,
        value.values,
        value.unit,
        value.scale,
        value.stats.map(encode_stats_bitmap).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_reading_bitmap(bytes: &[u8]) -> Result<Reading> {
    let record = generated::ReadingBitmap::deserialize_checked(bytes)?;
    Ok(Reading {
        scale: record.scale(),
        sensor_id: record.sensor_id,
        values: record.values,
        unit: record.unit,
        stats: record
            .stats
            .as_deref()
            .map(decode_stats_bitmap)
            .transpose()?,
    })
}

fn encode_limits_bitmap(value: Limits) -> Result<Vec<u8>> {
    let record = generated::LimitsBitmap::try_new(value.low, value.high)?;
    Ok(record.serialize_checked()?)
}
fn decode_limits_bitmap(bytes: &[u8]) -> Result<Limits> {
    let record = generated::LimitsBitmap::deserialize_checked(bytes)?;
    Ok(Limits {
        low: record.low(),
        high: record.high(),
    })
}

fn encode_patch_bitmap(value: Patch) -> Result<Vec<u8>> {
    let record = generated::PatchBitmap::try_new(
        value.firmware,
        value.enabled.map(u8::from),
        value.limits.map(encode_limits_bitmap).transpose()?,
        value.reboot_after_ms,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_patch_bitmap(bytes: &[u8]) -> Result<Patch> {
    let record = generated::PatchBitmap::deserialize_checked(bytes)?;
    Ok(Patch {
        enabled: record.enabled().map(|item| item != 0),
        reboot_after_ms: record.reboot_after_ms(),
        firmware: record.firmware,
        limits: record
            .limits
            .as_deref()
            .map(decode_limits_bitmap)
            .transpose()?,
    })
}

fn encode_update_bitmap(value: Update) -> Result<Vec<u8>> {
    let record = generated::UpdateBitmap::try_new(
        value.entity_id,
        value.mask,
        value.patch.map(encode_patch_bitmap).transpose()?,
        value.generation,
        value.reason,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_update_bitmap(bytes: &[u8]) -> Result<Update> {
    let record = generated::UpdateBitmap::deserialize_checked(bytes)?;
    Ok(Update {
        generation: record.generation(),
        entity_id: record.entity_id,
        mask: record.mask,
        patch: record
            .patch
            .as_deref()
            .map(decode_patch_bitmap)
            .transpose()?,
        reason: record.reason,
    })
}

fn encode_retry_bitmap(value: Retry) -> Result<Vec<u8>> {
    let record = generated::RetryBitmap::try_new(value.attempt, value.backoff_ms, value.token)?;
    Ok(record.serialize_checked()?)
}
fn decode_retry_bitmap(bytes: &[u8]) -> Result<Retry> {
    let record = generated::RetryBitmap::deserialize_checked(bytes)?;
    Ok(Retry {
        attempt: record.attempt(),
        backoff_ms: record.backoff_ms(),
        token: record.token,
    })
}

fn encode_notice_bitmap(value: Notice) -> Result<Vec<u8>> {
    let record = generated::NoticeBitmap {
        severity: value.severity,
        code: value.code,
        related: value.related,
        summary: value.summary,
        retry: value.retry.map(encode_retry_bitmap).transpose()?,
    };
    Ok(record.serialize_checked()?)
}
fn decode_notice_bitmap(bytes: &[u8]) -> Result<Notice> {
    let record = generated::NoticeBitmap::deserialize_checked(bytes)?;
    Ok(Notice {
        severity: record.severity,
        code: record.code,
        related: record.related,
        summary: record.summary,
        retry: record
            .retry
            .as_deref()
            .map(decode_retry_bitmap)
            .transpose()?,
    })
}

fn encode_heartbeat_bitmap(value: Heartbeat) -> Result<Vec<u8>> {
    let record =
        generated::HeartbeatBitmap::try_new(value.sequence, value.uptime_ms, value.lag_ms)?;
    Ok(record.serialize_checked()?)
}
fn decode_heartbeat_bitmap(bytes: &[u8]) -> Result<Heartbeat> {
    let record = generated::HeartbeatBitmap::deserialize_checked(bytes)?;
    Ok(Heartbeat {
        lag_ms: record.lag_ms(),
        sequence: record.sequence,
        uptime_ms: record.uptime_ms,
    })
}

fn encode_event_bitmap(value: Event) -> Result<Vec<u8>> {
    let record = generated::EventBitmap {
        sequence: value.sequence,
        topic: value.topic,
        affected: value.affected,
        note: value.note,
        retry: value.retry.map(encode_retry_bitmap).transpose()?,
        patch: value.patch.map(encode_patch_bitmap).transpose()?,
    };
    Ok(record.serialize_checked()?)
}
fn decode_event_bitmap(bytes: &[u8]) -> Result<Event> {
    let record = generated::EventBitmap::deserialize_checked(bytes)?;
    Ok(Event {
        sequence: record.sequence,
        topic: record.topic,
        affected: record.affected,
        note: record.note,
        retry: record
            .retry
            .as_deref()
            .map(decode_retry_bitmap)
            .transpose()?,
        patch: record
            .patch
            .as_deref()
            .map(decode_patch_bitmap)
            .transpose()?,
    })
}

fn encode_envelope_bitmap(value: Envelope) -> Result<Vec<u8>> {
    let (kind, payload) = match value.body.ok_or("missing body")? {
        Body::Reading(child) => (1, encode_reading_bitmap(child)?),
        Body::Update(child) => (2, encode_update_bitmap(child)?),
        Body::Notice(child) => (3, encode_notice_bitmap(child)?),
        Body::Heartbeat(child) => (4, encode_heartbeat_bitmap(child)?),
        Body::Event(child) => (5, encode_event_bitmap(child)?),
    };
    let record = generated::EnvelopeBitmap::try_new(
        value.request_id,
        value.revision,
        value.timestamp_ms,
        kind,
        payload,
        value.deadline_ms,
        value.device_id,
        value.tracing.map(u8::from),
        value.trace,
        value.context.map(encode_context_bitmap).transpose()?,
    )?;
    Ok(record.serialize_checked()?)
}
fn decode_envelope_bitmap(bytes: &[u8]) -> Result<Envelope> {
    let record = generated::EnvelopeBitmap::deserialize_checked(bytes)?;
    Ok(Envelope {
        deadline_ms: record.deadline_ms(),
        device_id: record.device_id(),
        tracing: record.tracing().map(|item| item != 0),
        request_id: record.request_id,
        revision: record.revision,
        timestamp_ms: record.timestamp_ms,
        trace: record.trace,
        context: record
            .context
            .as_deref()
            .map(decode_context_bitmap)
            .transpose()?,
        body: Some(match record.kind {
            1 => Body::Reading(decode_reading_bitmap(&record.payload)?),
            2 => Body::Update(decode_update_bitmap(&record.payload)?),
            3 => Body::Notice(decode_notice_bitmap(&record.payload)?),
            4 => Body::Heartbeat(decode_heartbeat_bitmap(&record.payload)?),
            5 => Body::Event(decode_event_bitmap(&record.payload)?),
            _ => return Err("unknown QSL body".into()),
        }),
    })
}

pub fn encode(value: Envelope, bitmap: bool) -> Result<Vec<u8>> {
    if bitmap {
        encode_envelope_bitmap(value)
    } else {
        encode_envelope_plain(value)
    }
}
pub fn decode(bytes: &[u8], bitmap: bool) -> Result<Envelope> {
    if bitmap {
        decode_envelope_bitmap(bytes)
    } else {
        decode_envelope_plain(bytes)
    }
}
