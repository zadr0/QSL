use crate::model::*;
pub const RECORDS: usize = 256;
pub const WORKLOADS: [&str; 3] = ["api", "heartbeat", "event"];

pub fn make(workload: &str, index: usize) -> Envelope {
    let i = index as u32;
    let n = ((i + 53).wrapping_mul(0xa581_f641)).rotate_left(11);
    let base = Envelope {
        request_id: 20_000 + i,
        revision: i % 8,
        timestamp_ms: 1_790_000_000_000 + i as u64 * 73,
        deadline_ms: None,
        device_id: None,
        tracing: None,
        trace: None,
        context: None,
        body: None,
    };
    let patch = || Patch {
        firmware: (i & 1 != 0).then(|| {
            if i % 7 == 0 {
                String::new()
            } else {
                format!("edge-2.{}.{}", i % 12, i % 17)
            }
        }),
        enabled: (i & 2 == 0).then_some(i & 8 != 0),
        limits: (i % 3 != 0).then(|| Limits {
            low: (i & 4 == 0).then_some(i % 6),
            high: (i & 16 != 0).then_some(n & 4095),
        }),
        reboot_after_ms: (i % 11 == 0).then_some(i % 500),
    };
    let retry = || Retry {
        attempt: (i % 3 != 0).then_some(i % 5),
        backoff_ms: (i % 5 != 0).then_some((i % 12) * 50),
        token: (i & 8 != 0).then(|| format!("replay-{n:08x}")),
    };
    let values = || (0..(i % 24)).map(|j| n.rotate_left(j) & 65535).collect();
    let body = match workload {
        "api" => match i % 8 {
            0..=4 => Body::Reading(Reading {
                sensor_id: n % 4096,
                values: values(),
                unit: (i % 5 != 0).then(|| "μg/m³".to_string()),
                scale: (i & 8 != 0).then_some(i % 4),
                stats: (i % 3 == 0).then(|| Stats {
                    minimum: i % 19,
                    maximum: n & 65535,
                    error_count: (i % 5 == 0).then_some(i % 2),
                }),
            }),
            5..=6 => Body::Update(Update {
                entity_id: n,
                mask: i % 64,
                patch: (i % 5 != 0).then(patch),
                generation: (i & 4 != 0).then_some(i % 8),
                reason: (i % 3 == 0).then(|| format!("Плановая настройка узла {i}")),
            }),
            _ => Body::Notice(Notice {
                severity: i % 5,
                code: 400 + i % 100,
                related: values(),
                summary: (i % 3 != 0).then(|| "Устройство временно недоступно".to_string()),
                retry: (i % 5 != 0).then(retry),
            }),
        },
        "heartbeat" => Body::Heartbeat(Heartbeat {
            sequence: i,
            uptime_ms: 8_640_000 + i as u64 * 5_000,
            lag_ms: (i % 11 == 0).then_some(i % 14),
        }),
        "event" => Body::Event(Event {
            sequence: i,
            topic: i % 9,
            affected: values(),
            note: (i % 3 == 0).then(|| format!("Обновлён датчик {i}")),
            retry: (i % 7 == 0).then(retry),
            patch: (i % 5 == 0).then(patch),
        }),
        _ => panic!("unknown workload {workload}"),
    };
    let mut envelope = Envelope {
        body: Some(body),
        ..base
    };
    if workload == "heartbeat" {
        return envelope;
    }
    envelope.device_id = (i % 11 != 0).then_some(n % 8192);
    envelope.trace = (i % 7 == 0).then(|| {
        if i % 49 == 0 {
            String::new()
        } else {
            format!("{n:08x}-{i:08x}")
        }
    });
    envelope.tracing = (i & 1 != 0).then_some(i & 8 != 0);
    envelope.deadline_ms = (i % 4 != 0).then_some(i % 300);
    envelope.context = (i % 5 != 1).then(|| Context {
        gateway: (i % 2 == 0).then_some(n % 256),
        label: (i % 4 == 0).then(|| format!("Станция {i}")),
        geo: (i % 3 != 0).then(|| Geo {
            zone: (i % 11 == 0).then_some(i % 2),
            city: (i % 7 == 0).then(|| "Екатеринбург".to_string()),
            altitude_m: (i % 2 == 1).then_some(n % 3100),
        }),
        auth: (i % 7 != 0).then(|| Auth {
            session_id: (i % 6 == 0).then_some(n as u64 * 1000),
            scope: (i % 9 == 0).then(|| String::from("telemetry:write")),
            quota: (i % 2 == 0).then_some(i % 4),
        }),
    });
    envelope
}
