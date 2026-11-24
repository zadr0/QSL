import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const port = Number(process.argv[2] ?? '7360');
const reports = new Set(['dashboard.html','dashboard.en.html','historical-dashboard.html','report.html','qsl-matrix.html','README.md','OPTIMIZED.md','HISTORY.md','QSL-ALL-FEATURES.md','QSL-SAFETY.md','RESULTS.md','SAME-STRUCTURES.md',
  'REALISTIC.md','QSL-FEATURES.md','QSL-ONLY.md','DYN.md','LAYOUT.md','K6.md']);
const data = /^results\/(?:optimized(?:-batch)?|qsl-matrix|realistic|qsl-features|released|dyn|layout|k6)\/[a-z0-9_./-]+\.(?:csv|json|svg|qsl|gz)$/i;
const mime = {'.html':'text/html; charset=utf-8','.md':'text/plain; charset=utf-8',
  '.csv':'text/csv; charset=utf-8','.json':'application/json; charset=utf-8','.svg':'image/svg+xml','.qsl':'text/plain; charset=utf-8'};
await fs.access(path.join(root,'dashboard.html'));
const server = http.createServer(async (req, res) => {
  try {
    if (req.method !== 'GET' && req.method !== 'HEAD') { res.writeHead(405).end(); return; }
    const request = decodeURIComponent(new URL(req.url, 'http://127.0.0.1').pathname).slice(1) || 'dashboard.html';
    if (request.split('/').includes('..') || (!reports.has(request) && !data.test(request))) {
      res.writeHead(404).end('Report not found');
      return;
    }
    const file = path.join(root, request);
    const bytes = await fs.readFile(file);
    res.writeHead(200, {'Content-Type':mime[path.extname(file)] ?? 'application/octet-stream',
      'Content-Length':bytes.length,'Cache-Control':'no-store','X-Content-Type-Options':'nosniff'});
    res.end(req.method === 'HEAD' ? undefined : bytes);
  } catch {
    res.writeHead(404).end('Report not found');
  }
});
server.listen(port,'127.0.0.1',()=>console.log('Benchmark dashboard: http://127.0.0.1:'+port+'/dashboard.html'));
