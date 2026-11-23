use crate::{model::*, telemetry_capnp as schema, Result};
fn write_geo(mut builder: schema::geo::Builder<'_>, value: &Geo) {
    if let Some(item) = &value.zone {
        builder.set_has_zone(true);
        builder.set_zone(*item);
    }
    if let Some(item) = &value.city {
        builder.set_city(item.as_str());
    }
    if let Some(item) = &value.altitude_m {
        builder.set_has_altitude_m(true);
        builder.set_altitude_m(*item);
    }
}
fn read_geo(reader: schema::geo::Reader<'_>) -> Result<Geo> {
    Ok(Geo {
        zone: reader.get_has_zone().then(|| reader.get_zone()),
        city: if reader.has_city() {
            Some(reader.get_city()?.to_str()?.to_owned())
        } else {
            None
        },
        altitude_m: reader.get_has_altitude_m().then(|| reader.get_altitude_m()),
    })
}

fn write_auth(mut builder: schema::auth::Builder<'_>, value: &Auth) {
    if let Some(item) = &value.session_id {
        builder.set_has_session_id(true);
        builder.set_session_id(*item);
    }
    if let Some(item) = &value.scope {
        builder.set_scope(item.as_str());
    }
    if let Some(item) = &value.quota {
        builder.set_has_quota(true);
        builder.set_quota(*item);
    }
}
fn read_auth(reader: schema::auth::Reader<'_>) -> Result<Auth> {
    Ok(Auth {
        session_id: reader.get_has_session_id().then(|| reader.get_session_id()),
        scope: if reader.has_scope() {
            Some(reader.get_scope()?.to_str()?.to_owned())
        } else {
            None
        },
        quota: reader.get_has_quota().then(|| reader.get_quota()),
    })
}

fn write_context(mut builder: schema::context::Builder<'_>, value: &Context) {
    if let Some(item) = &value.gateway {
        builder.set_has_gateway(true);
        builder.set_gateway(*item);
    }
    if let Some(item) = &value.label {
        builder.set_label(item.as_str());
    }
    if let Some(item) = &value.geo {
        write_geo(builder.reborrow().init_geo(), item);
    }
    if let Some(item) = &value.auth {
        write_auth(builder.reborrow().init_auth(), item);
    }
}
fn read_context(reader: schema::context::Reader<'_>) -> Result<Context> {
    Ok(Context {
        gateway: reader.get_has_gateway().then(|| reader.get_gateway()),
        label: if reader.has_label() {
            Some(reader.get_label()?.to_str()?.to_owned())
        } else {
            None
        },
        geo: if reader.has_geo() {
            Some(read_geo(reader.get_geo()?)?)
        } else {
            None
        },
        auth: if reader.has_auth() {
            Some(read_auth(reader.get_auth()?)?)
        } else {
            None
        },
    })
}

fn write_stats(mut builder: schema::stats::Builder<'_>, value: &Stats) {
    builder.set_minimum(value.minimum);
    builder.set_maximum(value.maximum);
    if let Some(item) = &value.error_count {
        builder.set_has_error_count(true);
        builder.set_error_count(*item);
    }
}
fn read_stats(reader: schema::stats::Reader<'_>) -> Result<Stats> {
    Ok(Stats {
        minimum: reader.get_minimum(),
        maximum: reader.get_maximum(),
        error_count: reader
            .get_has_error_count()
            .then(|| reader.get_error_count()),
    })
}

fn write_reading(mut builder: schema::reading::Builder<'_>, value: &Reading) {
    builder.set_sensor_id(value.sensor_id);
    let mut list = builder.reborrow().init_values(value.values.len() as u32);
    for (index, &item) in value.values.iter().enumerate() {
        list.set(index as u32, item);
    }
    if let Some(item) = &value.unit {
        builder.set_unit(item.as_str());
    }
    if let Some(item) = &value.scale {
        builder.set_has_scale(true);
        builder.set_scale(*item);
    }
    if let Some(item) = &value.stats {
        write_stats(builder.reborrow().init_stats(), item);
    }
}
fn read_reading(reader: schema::reading::Reader<'_>) -> Result<Reading> {
    Ok(Reading {
        sensor_id: reader.get_sensor_id(),
        values: reader.get_values()?.iter().collect(),
        unit: if reader.has_unit() {
            Some(reader.get_unit()?.to_str()?.to_owned())
        } else {
            None
        },
        scale: reader.get_has_scale().then(|| reader.get_scale()),
        stats: if reader.has_stats() {
            Some(read_stats(reader.get_stats()?)?)
        } else {
            None
        },
    })
}

fn write_limits(mut builder: schema::limits::Builder<'_>, value: &Limits) {
    if let Some(item) = &value.low {
        builder.set_has_low(true);
        builder.set_low(*item);
    }
    if let Some(item) = &value.high {
        builder.set_has_high(true);
        builder.set_high(*item);
    }
}
fn read_limits(reader: schema::limits::Reader<'_>) -> Result<Limits> {
    Ok(Limits {
        low: reader.get_has_low().then(|| reader.get_low()),
        high: reader.get_has_high().then(|| reader.get_high()),
    })
}

fn write_patch(mut builder: schema::patch::Builder<'_>, value: &Patch) {
    if let Some(item) = &value.firmware {
        builder.set_firmware(item.as_str());
    }
    if let Some(item) = &value.enabled {
        builder.set_has_enabled(true);
        builder.set_enabled(*item);
    }
    if let Some(item) = &value.limits {
        write_limits(builder.reborrow().init_limits(), item);
    }
    if let Some(item) = &value.reboot_after_ms {
        builder.set_has_reboot_after_ms(true);
        builder.set_reboot_after_ms(*item);
    }
}
fn read_patch(reader: schema::patch::Reader<'_>) -> Result<Patch> {
    Ok(Patch {
        firmware: if reader.has_firmware() {
            Some(reader.get_firmware()?.to_str()?.to_owned())
        } else {
            None
        },
        enabled: reader.get_has_enabled().then(|| reader.get_enabled()),
        limits: if reader.has_limits() {
            Some(read_limits(reader.get_limits()?)?)
        } else {
            None
        },
        reboot_after_ms: reader
            .get_has_reboot_after_ms()
            .then(|| reader.get_reboot_after_ms()),
    })
}

fn write_update(mut builder: schema::update::Builder<'_>, value: &Update) {
    builder.set_entity_id(value.entity_id);
    builder.set_mask(value.mask);
    if let Some(item) = &value.patch {
        write_patch(builder.reborrow().init_patch(), item);
    }
    if let Some(item) = &value.generation {
        builder.set_has_generation(true);
        builder.set_generation(*item);
    }
    if let Some(item) = &value.reason {
        builder.set_reason(item.as_str());
    }
}
fn read_update(reader: schema::update::Reader<'_>) -> Result<Update> {
    Ok(Update {
        entity_id: reader.get_entity_id(),
        mask: reader.get_mask(),
        patch: if reader.has_patch() {
            Some(read_patch(reader.get_patch()?)?)
        } else {
            None
        },
        generation: reader.get_has_generation().then(|| reader.get_generation()),
        reason: if reader.has_reason() {
            Some(reader.get_reason()?.to_str()?.to_owned())
        } else {
            None
        },
    })
}

fn write_retry(mut builder: schema::retry::Builder<'_>, value: &Retry) {
    if let Some(item) = &value.attempt {
        builder.set_has_attempt(true);
        builder.set_attempt(*item);
    }
    if let Some(item) = &value.backoff_ms {
        builder.set_has_backoff_ms(true);
        builder.set_backoff_ms(*item);
    }
    if let Some(item) = &value.token {
        builder.set_token(item.as_str());
    }
}
fn read_retry(reader: schema::retry::Reader<'_>) -> Result<Retry> {
    Ok(Retry {
        attempt: reader.get_has_attempt().then(|| reader.get_attempt()),
        backoff_ms: reader.get_has_backoff_ms().then(|| reader.get_backoff_ms()),
        token: if reader.has_token() {
            Some(reader.get_token()?.to_str()?.to_owned())
        } else {
            None
        },
    })
}

fn write_notice(mut builder: schema::notice::Builder<'_>, value: &Notice) {
    builder.set_severity(value.severity);
    builder.set_code(value.code);
    let mut list = builder.reborrow().init_related(value.related.len() as u32);
    for (index, &item) in value.related.iter().enumerate() {
        list.set(index as u32, item);
    }
    if let Some(item) = &value.summary {
        builder.set_summary(item.as_str());
    }
    if let Some(item) = &value.retry {
        write_retry(builder.reborrow().init_retry(), item);
    }
}
fn read_notice(reader: schema::notice::Reader<'_>) -> Result<Notice> {
    Ok(Notice {
        severity: reader.get_severity(),
        code: reader.get_code(),
        related: reader.get_related()?.iter().collect(),
        summary: if reader.has_summary() {
            Some(reader.get_summary()?.to_str()?.to_owned())
        } else {
            None
        },
        retry: if reader.has_retry() {
            Some(read_retry(reader.get_retry()?)?)
        } else {
            None
        },
    })
}

fn write_heartbeat(mut builder: schema::heartbeat::Builder<'_>, value: &Heartbeat) {
    builder.set_sequence(value.sequence);
    builder.set_uptime_ms(value.uptime_ms);
    if let Some(item) = &value.lag_ms {
        builder.set_has_lag_ms(true);
        builder.set_lag_ms(*item);
    }
}
fn read_heartbeat(reader: schema::heartbeat::Reader<'_>) -> Result<Heartbeat> {
    Ok(Heartbeat {
        sequence: reader.get_sequence(),
        uptime_ms: reader.get_uptime_ms(),
        lag_ms: reader.get_has_lag_ms().then(|| reader.get_lag_ms()),
    })
}

fn write_event(mut builder: schema::event::Builder<'_>, value: &Event) {
    builder.set_sequence(value.sequence);
    builder.set_topic(value.topic);
    let mut list = builder
        .reborrow()
        .init_affected(value.affected.len() as u32);
    for (index, &item) in value.affected.iter().enumerate() {
        list.set(index as u32, item);
    }
    if let Some(item) = &value.note {
        builder.set_note(item.as_str());
    }
    if let Some(item) = &value.retry {
        write_retry(builder.reborrow().init_retry(), item);
    }
    if let Some(item) = &value.patch {
        write_patch(builder.reborrow().init_patch(), item);
    }
}
fn read_event(reader: schema::event::Reader<'_>) -> Result<Event> {
    Ok(Event {
        sequence: reader.get_sequence(),
        topic: reader.get_topic(),
        affected: reader.get_affected()?.iter().collect(),
        note: if reader.has_note() {
            Some(reader.get_note()?.to_str()?.to_owned())
        } else {
            None
        },
        retry: if reader.has_retry() {
            Some(read_retry(reader.get_retry()?)?)
        } else {
            None
        },
        patch: if reader.has_patch() {
            Some(read_patch(reader.get_patch()?)?)
        } else {
            None
        },
    })
}

fn write_envelope(mut builder: schema::envelope::Builder<'_>, value: &Envelope) {
    builder.set_request_id(value.request_id);
    builder.set_revision(value.revision);
    builder.set_timestamp_ms(value.timestamp_ms);
    if let Some(item) = &value.deadline_ms {
        builder.set_has_deadline_ms(true);
        builder.set_deadline_ms(*item);
    }
    if let Some(item) = &value.device_id {
        builder.set_has_device_id(true);
        builder.set_device_id(*item);
    }
    if let Some(item) = &value.tracing {
        builder.set_has_tracing(true);
        builder.set_tracing(*item);
    }
    if let Some(item) = &value.trace {
        builder.set_trace(item.as_str());
    }
    if let Some(item) = &value.context {
        write_context(builder.reborrow().init_context(), item);
    }
    let body = builder.reborrow().get_body();
    match value.body.as_ref() {
        Some(Body::Reading(child)) => write_reading(body.init_reading(), child),
        Some(Body::Update(child)) => write_update(body.init_update(), child),
        Some(Body::Notice(child)) => write_notice(body.init_notice(), child),
        Some(Body::Heartbeat(child)) => write_heartbeat(body.init_heartbeat(), child),
        Some(Body::Event(child)) => write_event(body.init_event(), child),
        None => {
            let mut body = body;
            body.set_none(());
        }
    }
}
fn read_envelope(reader: schema::envelope::Reader<'_>) -> Result<Envelope> {
    Ok(Envelope {
        request_id: reader.get_request_id(),
        revision: reader.get_revision(),
        timestamp_ms: reader.get_timestamp_ms(),
        deadline_ms: reader
            .get_has_deadline_ms()
            .then(|| reader.get_deadline_ms()),
        device_id: reader.get_has_device_id().then(|| reader.get_device_id()),
        tracing: reader.get_has_tracing().then(|| reader.get_tracing()),
        trace: if reader.has_trace() {
            Some(reader.get_trace()?.to_str()?.to_owned())
        } else {
            None
        },
        context: if reader.has_context() {
            Some(read_context(reader.get_context()?)?)
        } else {
            None
        },
        body: match reader.get_body().which()? {
            schema::envelope::body::None(()) => None,
            schema::envelope::body::Reading(child) => Some(Body::Reading(read_reading(child?)?)),
            schema::envelope::body::Update(child) => Some(Body::Update(read_update(child?)?)),
            schema::envelope::body::Notice(child) => Some(Body::Notice(read_notice(child?)?)),
            schema::envelope::body::Heartbeat(child) => {
                Some(Body::Heartbeat(read_heartbeat(child?)?))
            }
            schema::envelope::body::Event(child) => Some(Body::Event(read_event(child?)?)),
        },
    })
}

pub fn encode(value: &Envelope, packed: bool) -> Result<Vec<u8>> {
    let mut message = capnp::message::Builder::new_default();
    write_envelope(message.init_root::<schema::envelope::Builder<'_>>(), value);
    if packed {
        let mut output = Vec::new();
        capnp::serialize_packed::write_message(&mut output, &message)?;
        Ok(output)
    } else {
        Ok(capnp::serialize::write_message_to_words(&message))
    }
}

pub fn decode(mut bytes: &[u8], packed: bool) -> Result<Envelope> {
    let options = capnp::message::ReaderOptions {
        traversal_limit_in_words: Some(32_768),
        nesting_limit: 32,
    };
    let value = if packed {
        let reader = capnp::serialize_packed::read_message(&mut bytes, options)?;
        read_envelope(reader.get_root::<schema::envelope::Reader<'_>>()?)?
    } else {
        let reader = capnp::serialize::read_message_from_flat_slice_no_alloc(&mut bytes, options)?;
        read_envelope(reader.get_root::<schema::envelope::Reader<'_>>()?)?
    };
    if !bytes.is_empty() {
        return Err("trailing Cap'n Proto bytes".into());
    }
    Ok(value)
}
