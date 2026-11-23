@0xaee6f12ac1de968b;
struct Geo {
  hasZone @0 :Bool;
  zone @1 :UInt32;
  city @2 :Text;
  hasAltitudeM @3 :Bool;
  altitudeM @4 :UInt32;
}

struct Auth {
  hasSessionId @0 :Bool;
  sessionId @1 :UInt64;
  scope @2 :Text;
  hasQuota @3 :Bool;
  quota @4 :UInt32;
}

struct Context {
  hasGateway @0 :Bool;
  gateway @1 :UInt32;
  label @2 :Text;
  geo @3 :Geo;
  auth @4 :Auth;
}

struct Stats {
  minimum @0 :UInt32;
  maximum @1 :UInt32;
  hasErrorCount @2 :Bool;
  errorCount @3 :UInt32;
}

struct Reading {
  sensorId @0 :UInt32;
  values @1 :List(UInt32);
  unit @2 :Text;
  hasScale @3 :Bool;
  scale @4 :UInt32;
  stats @5 :Stats;
}

struct Limits {
  hasLow @0 :Bool;
  low @1 :UInt32;
  hasHigh @2 :Bool;
  high @3 :UInt32;
}

struct Patch {
  firmware @0 :Text;
  hasEnabled @1 :Bool;
  enabled @2 :Bool;
  limits @3 :Limits;
  hasRebootAfterMs @4 :Bool;
  rebootAfterMs @5 :UInt32;
}

struct Update {
  entityId @0 :UInt32;
  mask @1 :UInt32;
  patch @2 :Patch;
  hasGeneration @3 :Bool;
  generation @4 :UInt32;
  reason @5 :Text;
}

struct Retry {
  hasAttempt @0 :Bool;
  attempt @1 :UInt32;
  hasBackoffMs @2 :Bool;
  backoffMs @3 :UInt32;
  token @4 :Text;
}

struct Notice {
  severity @0 :UInt32;
  code @1 :UInt32;
  related @2 :List(UInt32);
  summary @3 :Text;
  retry @4 :Retry;
}

struct Heartbeat {
  sequence @0 :UInt32;
  uptimeMs @1 :UInt64;
  hasLagMs @2 :Bool;
  lagMs @3 :UInt32;
}

struct Event {
  sequence @0 :UInt32;
  topic @1 :UInt32;
  affected @2 :List(UInt32);
  note @3 :Text;
  retry @4 :Retry;
  patch @5 :Patch;
}

struct Envelope {
  requestId @0 :UInt32;
  revision @1 :UInt32;
  timestampMs @2 :UInt64;
  hasDeadlineMs @3 :Bool;
  deadlineMs @4 :UInt32;
  hasDeviceId @5 :Bool;
  deviceId @6 :UInt32;
  hasTracing @7 :Bool;
  tracing @8 :Bool;
  trace @9 :Text;
  context @10 :Context;
  body :union {
    none @11 :Void;
    reading @12 :Reading;
    update @13 :Update;
    notice @14 :Notice;
    heartbeat @15 :Heartbeat;
    event @16 :Event;
  }
}

