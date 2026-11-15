import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { existsSync } from 'node:fs';
const root=resolve(import.meta.dirname,'../..');
export const analyzer=resolve(root,'.optimization/unified-target/debug/qsl-analyze.exe');
export const examples=resolve(root,'.optimization/docs-site-target/debug/qsl-docs-examples.exe');
export function invoke(executable,args,input){return new Promise((resolve,reject)=>{
 if(!existsSync(executable)){reject(new Error('Native example engine is not built. Run npm run native:build.'));return;}
 const child=spawn(executable,args,{windowsHide:true,stdio:['pipe','pipe','pipe'],shell:false});
 let output='',errors='',failed=false;
 const timer=setTimeout(()=>{failed=true;child.kill();reject(new Error('Analysis time limit exceeded'));},5000);
 child.stdout.on('data',d=>{output+=d;if(output.length>4*1024*1024){failed=true;child.kill();reject(new Error('Generated output limit exceeded'));}});
 child.stderr.on('data',d=>{if(errors.length<4096)errors+=d});
 child.on('error',e=>{clearTimeout(timer);reject(e)});
 child.on('close',code=>{clearTimeout(timer);if(failed)return;if(code){reject(new Error('Native process failed'));return;}try{resolve(JSON.parse(output))}catch{reject(new Error('Invalid native response'))}});
 child.stdin.on('error',()=>{});child.stdin.end(input);
});}
export default function nativeBridge(){let active=0;return {name:'qsl-native-examples',configureServer(server){server.middlewares.use('/api/qsl',async(req,res)=>{
 const endpoint=(req.url||'').replace(/^\/api\/qsl/,'').split('?')[0].replace(/\/+$/,'');
 res.setHeader('Content-Type','application/json; charset=utf-8');res.setHeader('Cache-Control','no-store');
 const send=(code,value)=>{res.statusCode=code;res.end(JSON.stringify(value))};
 if(req.method==='GET'&&endpoint==='/health'){send(200,{ok:existsSync(analyzer)&&existsSync(examples),analyzer:existsSync(analyzer),examples:existsSync(examples)});return;}
 if(req.method!=='POST'){send(405,{ok:false,error:'POST required'});return;}
 const host=req.headers.host||'';const origin=req.headers.origin;
 if(!/^(127\.0\.0\.1|localhost):\d+$/.test(host)||(origin&&new URL(origin).host!==host)){send(403,{ok:false,error:'Local same-origin requests only'});return;}
 if(active>=2){send(429,{ok:false,error:'Two examples are already running'});return;}
 let body='';try{for await(const chunk of req){body+=chunk;if(Buffer.byteLength(body)>65536){send(413,{ok:false,error:'Request exceeds 64 KiB'});return;}}
 const data=JSON.parse(body);active++;
 try{if(endpoint==='/analyze'){
  if(typeof data.source!=='string'||data.source.length>32768||typeof(data.config??'')!=='string'){send(400,{ok:false,error:'Invalid schema request'});return;}
  const args=['--generate','--deq-text',data.config||''];send(200,await invoke(analyzer,args,data.source));
 }else if(endpoint==='/run'){send(200,await invoke(examples,[],JSON.stringify(data)))}else{send(404,{ok:false,error:'Unknown operation'})}
 }finally{active--}
 }catch(error){send(400,{ok:false,error:error.message})}
 });}}}
