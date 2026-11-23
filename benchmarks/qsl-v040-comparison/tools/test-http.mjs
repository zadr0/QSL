import fs from 'node:fs/promises';
import http from 'node:http';
import assert from 'node:assert/strict';
const [base,folder]=process.argv.slice(2);
const manifest=JSON.parse(await fs.readFile(folder+'/manifest.json','utf8'));
const agent=new http.Agent({keepAlive:true,maxSockets:1});
const request=(route,method,body)=>new Promise((resolve,reject)=>{
 const r=http.request(base+route,{agent,method,headers:{'Content-Length':body?.length??0}},s=>{const chunks=[];s.on('data',c=>chunks.push(c));s.on('end',()=>resolve({status:s.statusCode,body:Buffer.concat(chunks)}));});
 r.on('error',reject);r.end(body);
});
let count=0;
try{
 for(const profile of manifest){
  const blob=await fs.readFile(folder+'/'+profile.filename);
  for(const i of [0,7,255]){
   const record=profile.records[i],frame=blob.subarray(record.offset,record.offset+record.length);
   const read=await request('/decode/'+profile.id+'/'+i,'POST',frame);
   assert.equal(read.status,200);assert.equal(read.body.toString(),record.digest);
   const write=await request('/encode/'+profile.id+'/'+i,'GET');
   assert.equal(write.status,200);assert.deepEqual(write.body,frame);count+=2;
  }
  const bad=await request('/decode/'+profile.id+'/0','POST',Buffer.from([0xff]));
  assert.equal(bad.status,400);
 }
 assert.equal((await request('/decode/unknown/unknown/0','POST',Buffer.alloc(0))).status,404);
 assert.equal((await request('/encode/numeric/qsl_default/256','GET')).status,400);
 console.log('HTTP contracts passed:',manifest.length,'profiles,',count,'byte/value round trips; malformed frames rejected');
}finally{agent.destroy();}
