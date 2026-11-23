#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Geo {
    #[prost(uint32, optional, tag = "1")]
    pub zone: Option<u32>,
    #[prost(string, optional, tag = "2")]
    pub city: Option<String>,
    #[prost(uint32, optional, tag = "3")]
    pub altitude_m: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Auth {
    #[prost(uint64, optional, tag = "1")]
    pub session_id: Option<u64>,
    #[prost(string, optional, tag = "2")]
    pub scope: Option<String>,
    #[prost(uint32, optional, tag = "3")]
    pub quota: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Context {
    #[prost(uint32, optional, tag = "1")]
    pub gateway: Option<u32>,
    #[prost(string, optional, tag = "2")]
    pub label: Option<String>,
    #[prost(message, optional, tag = "3")]
    pub geo: Option<Geo>,
    #[prost(message, optional, tag = "4")]
    pub auth: Option<Auth>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Stats {
    #[prost(uint32, tag = "1")]
    pub minimum: u32,
    #[prost(uint32, tag = "2")]
    pub maximum: u32,
    #[prost(uint32, optional, tag = "3")]
    pub error_count: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Reading {
    #[prost(uint32, tag = "1")]
    pub sensor_id: u32,
    #[prost(uint32, repeated, packed = "true", tag = "2")]
    pub values: Vec<u32>,
    #[prost(string, optional, tag = "3")]
    pub unit: Option<String>,
    #[prost(uint32, optional, tag = "4")]
    pub scale: Option<u32>,
    #[prost(message, optional, tag = "5")]
    pub stats: Option<Stats>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Limits {
    #[prost(uint32, optional, tag = "1")]
    pub low: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub high: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Patch {
    #[prost(string, optional, tag = "1")]
    pub firmware: Option<String>,
    #[prost(bool, optional, tag = "2")]
    pub enabled: Option<bool>,
    #[prost(message, optional, tag = "3")]
    pub limits: Option<Limits>,
    #[prost(uint32, optional, tag = "4")]
    pub reboot_after_ms: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Update {
    #[prost(uint32, tag = "1")]
    pub entity_id: u32,
    #[prost(uint32, tag = "2")]
    pub mask: u32,
    #[prost(message, optional, tag = "3")]
    pub patch: Option<Patch>,
    #[prost(uint32, optional, tag = "4")]
    pub generation: Option<u32>,
    #[prost(string, optional, tag = "5")]
    pub reason: Option<String>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Retry {
    #[prost(uint32, optional, tag = "1")]
    pub attempt: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub backoff_ms: Option<u32>,
    #[prost(string, optional, tag = "3")]
    pub token: Option<String>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Notice {
    #[prost(uint32, tag = "1")]
    pub severity: u32,
    #[prost(uint32, tag = "2")]
    pub code: u32,
    #[prost(uint32, repeated, packed = "true", tag = "3")]
    pub related: Vec<u32>,
    #[prost(string, optional, tag = "4")]
    pub summary: Option<String>,
    #[prost(message, optional, tag = "5")]
    pub retry: Option<Retry>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Heartbeat {
    #[prost(uint32, tag = "1")]
    pub sequence: u32,
    #[prost(uint64, tag = "2")]
    pub uptime_ms: u64,
    #[prost(uint32, optional, tag = "3")]
    pub lag_ms: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Event {
    #[prost(uint32, tag = "1")]
    pub sequence: u32,
    #[prost(uint32, tag = "2")]
    pub topic: u32,
    #[prost(uint32, repeated, packed = "true", tag = "3")]
    pub affected: Vec<u32>,
    #[prost(string, optional, tag = "4")]
    pub note: Option<String>,
    #[prost(message, optional, tag = "5")]
    pub retry: Option<Retry>,
    #[prost(message, optional, tag = "6")]
    pub patch: Option<Patch>,
}

#[derive(Clone, PartialEq, prost::Message, bincode::Encode, bincode::Decode)]
pub struct Envelope {
    #[prost(uint32, tag = "1")]
    pub request_id: u32,
    #[prost(uint32, tag = "2")]
    pub revision: u32,
    #[prost(uint64, tag = "3")]
    pub timestamp_ms: u64,
    #[prost(uint32, optional, tag = "4")]
    pub deadline_ms: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub device_id: Option<u32>,
    #[prost(bool, optional, tag = "6")]
    pub tracing: Option<bool>,
    #[prost(string, optional, tag = "7")]
    pub trace: Option<String>,
    #[prost(message, optional, tag = "8")]
    pub context: Option<Context>,
    #[prost(oneof = "Body", tags = "20, 21, 22, 23, 24")]
    pub body: Option<Body>,
}

#[derive(Clone, PartialEq, prost::Oneof, bincode::Encode, bincode::Decode)]
pub enum Body {
    #[prost(message, tag = "20")]
    Reading(Reading),
    #[prost(message, tag = "21")]
    Update(Update),
    #[prost(message, tag = "22")]
    Notice(Notice),
    #[prost(message, tag = "23")]
    Heartbeat(Heartbeat),
    #[prost(message, tag = "24")]
    Event(Event),
}
