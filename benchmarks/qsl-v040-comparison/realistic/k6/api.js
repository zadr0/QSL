import http from 'k6/http';
import exec from 'k6/execution';
import {Rate, Trend, Counter} from 'k6/metrics';

const folder = __ENV.FIXTURES;
const codec = __ENV.CODEC;
const profile = JSON.parse(open(folder + '/manifest.json')).find(item => item.codec === codec);
if (!profile) throw new Error('unknown fixture ' + codec);
const bytes = open(folder + '/' + profile.file, 'b');
const records = profile.records.map(({offset, request, response}) => ({
  request: bytes.slice(offset, offset + request),
  response: new Uint8Array(bytes, offset + request, response),
}));
export const options = {
  scenarios: {
    warmup: {executor: 'constant-vus', vus: 2, duration: '2s', gracefulStop: '1s'},
    measure: {executor: 'constant-vus', vus: 8, startTime: '3s', duration: '8s', gracefulStop: '2s'},
  },
  summaryTrendStats: ['min', 'avg', 'med', 'p(95)', 'p(99)', 'max'],
  thresholds: {verified: ['rate==1'], http_req_failed: ['rate==0']},
  systemTags: ['status', 'method', 'scenario', 'expected_response'],
};
const verified = new Rate('verified');
const reqs = new Counter('measured_requests');
const lat = new Trend('measured_latency_ms', true);
const bodyBytes = new Counter('measured_binary_body_bytes');
export default function () {
  const expected = records[exec.scenario.iterationInTest % records.length];
  const response = http.post(__ENV.BASE_URL + '/v1/normalize/' + codec, expected.request, {
    headers: {'Content-Type': 'application/octet-stream'}, responseType: 'binary', timeout: '5s',
  });
  const actual = new Uint8Array(response.body ?? new ArrayBuffer(0));
  let valid = response.status === 200 && actual.length === expected.response.length;
  for (let i = 0; valid && i < actual.length; i++) valid = actual[i] === expected.response[i];
  verified.add(valid);
  if (exec.scenario.name === 'measure') {
    reqs.add(1);
    lat.add(response.timings.duration);
    bodyBytes.add(expected.request.byteLength + actual.byteLength);
  }
}
export function handleSummary(data) {
  return {[__ENV.SUMMARY]: JSON.stringify({codec, measurement_seconds: 8, vus: 8, data}, null, 2)};
}

