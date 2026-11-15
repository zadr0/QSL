import {createServer} from 'node:http';
import {createReadStream,existsSync,statSync} from 'node:fs';
import {resolve,sep,extname} from 'node:path';
import nativeBridge from './native-bridge.mjs';
const root=resolve(import.meta.dirname,'../dist');
if(!existsSync(resolve(root,'ru/index.html')))throw new Error('Build the site first: npm run build');
let api;nativeBridge().configureServer({middlewares:{use(_prefix,handler){api=handler}}});
const types={'.html':'text/html; charset=utf-8','.css':'text/css; charset=utf-8','.js':'text/javascript; charset=utf-8','.mjs':'text/javascript; charset=utf-8','.json':'application/json; charset=utf-8','.svg':'image/svg+xml','.png':'image/png','.woff2':'font/woff2','.woff':'font/woff','.wasm':'application/wasm','.md':'text/plain; charset=utf-8','.rs':'text/plain; charset=utf-8','.xml':'application/xml; charset=utf-8'};
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://127.0.0.1:4322');
 if(url.pathname.startsWith('/api/qsl/')){req.url=url.pathname.slice('/api/qsl'.length);await api(req,res);return;}
 if(!['GET','HEAD'].includes(req.method)){res.writeHead(405);res.end();return;}
 const path=resolve(root,'.'+decodeURIComponent(url.pathname));
 if(path!==root&&!path.startsWith(root+sep)){res.writeHead(403);res.end();return;}
 let file=path;
 if(existsSync(file)&&statSync(file).isDirectory())file=resolve(file,'index.html');
 if(!existsSync(file)||!statSync(file).isFile()){res.writeHead(404,{'Content-Type':'text/plain; charset=utf-8'});res.end('Not found');return;}
 res.writeHead(200,{'Content-Type':types[extname(file)]||'application/octet-stream','Content-Length':statSync(file).size,'Cache-Control':'no-cache','X-Content-Type-Options':'nosniff'});
 if(req.method==='HEAD')res.end();else createReadStream(file).pipe(res);
 }catch(error){if(!res.headersSent)res.writeHead(400,{'Content-Type':'application/json'});res.end(JSON.stringify({ok:false,error:error.message}));}});
server.listen(4322,'127.0.0.1',()=>console.log('QSL documentation ready: http://127.0.0.1:4322/ru/'));
