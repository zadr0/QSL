import http from 'k6/http';
import {check} from 'k6';
import {Trend, Counter, Rate} from 'k6/metrics';
import exec from 'k6/execution';

const profileId = __ENV.PROFILE;
const operation = __ENV.OPERATION || 'decode';
const manifest = JSON.parse(open(__ENV.FIXTURES + '/manifest.json'));
const profile = manifest.find(p => p.id === profileId);
if (!profile) throw new Error('Unknown codec profile: ' + profileId);
const blob = open(__ENV.FIXTURES + '/' + profile.filename, 'b');
const duration = __ENV.DURATION || '10s';
const model = __ENV.MODEL || 'closed';
const measured = model === 'open' ? {
  executor: 'constant-arrival-rate', rate: Number(__ENV.RATE || 1000), timeUnit: '1s',
  preAllocatedVUs: 32, maxVUs: 32, duration,
} : {executor: 'constant-vus', vus: Number(__ENV.VUS || 8), duration};
export const options = {
  scenarios: {
    warmup: {executor: 'constant-vus', vus: 2, duration: '2s', gracefulStop: '1s'},
    measured: {...measured, startTime: '3s', gracefulStop: '3s'},
  },
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  thresholds: {verified: ['rate==1'], http_req_failed: ['rate==0']},
  systemTags: ['status', 'method', 'scenario', 'expected_response'],
  noConnectionReuse: false,
};
const latency = new Trend('measured_latency_ms', true);
const requests = new Counter('measured_requests');
const verified = new Rate('verified');
const verificationFailures = new Counter('measured_verification_failures');

export default function () {
  const index = exec.scenario.iterationInTest % profile.records.length;
  const record = profile.records[index];
  const input = blob.slice(record.offset, record.offset + record.length);
  const isEncode = operation.endsWith('encode');
  const url = (__ENV.BASE_URL || 'http://127.0.0.1:7340') + '/' + operation + '/' + profileId + '/' + index;
  const response = isEncode ? http.get(url, {responseType: 'binary', timeout: '5s'})
    : http.post(url, input, {responseType: 'text', headers: {'Content-Type': 'application/octet-stream'}, timeout: '5s'});
  let valid = response.status === 200;
  if (valid && isEncode) {
    const actual = new Uint8Array(response.body), expected = new Uint8Array(input);
    valid = actual.length === expected.length;
    for (let i = 0; valid && i < actual.length; i++) valid = actual[i] === expected[i];
  } else if (valid) valid = response.body === record.digest;
  check(valid, {'all bytes / fields verified': value => value});
  verified.add(valid);
  if (exec.scenario.name === 'measured') {
    latency.add(response.timings.duration);
    requests.add(1);
    verificationFailures.add(valid ? 0 : 1);
  }
}
export function handleSummary(data) {
  const output = {profile: profileId, operation, model, duration,
    vus: model === 'closed' ? Number(__ENV.VUS || 8) : null,
    requestedRate: model === 'open' ? Number(__ENV.RATE || 1000) : null,
    fixtureCount: profile.records.length, data};
  return {[__ENV.SUMMARY]: JSON.stringify(output, null, 2)};
}
