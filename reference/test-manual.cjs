const assert=require('node:assert/strict');const fs=require('node:fs');const api=require('../docs/assets/manual.js');
assert.deepEqual(api.plan('String',255,256),{kind:'String',object:24,heap:255,borrow:0,fallback:false});
assert.equal(api.plan('&str',2048,256).object,16);assert.equal(api.plan('&str',2048,256).heap,0);
for(const kind of ['StackString','InlineString']){assert.equal(api.plan(kind,255,256).kind,kind);assert.equal(api.plan(kind,255,256).object,260);assert.equal(api.plan(kind,257,256).kind,'String');assert.equal(api.plan(kind,2048,2048).object,2052);}
for(const a of [false,true])for(const b of [false,true])for(const invalid of [false,true])for(const table of [false,true]){const f=api.frame(a,b,invalid,table);assert.equal(f.valid,!invalid);assert.deepEqual(f.wire,[128*Number(a)+64*Number(b)+Number(invalid),9,...(b?[7]:[]),...(a?[52,18]:[])]);if(!invalid)assert.deepEqual(f.indices,[...(b?[2]:[]),...(a?[1]:[])]);}
for(const file of ['index.html','en.html']){const text=fs.readFileSync('docs/'+file,'utf8');for(const id of [...fs.readFileSync('docs/assets/manual.js','utf8').matchAll(/\$\('([a-z][a-z0-9-]+)'\)/g)].map(x=>x[1]))assert.ok(text.includes('id="'+id+'"'),file+': '+id);assert.ok(!text.includes('assets/full.js'));assert.ok(!text.includes('id="feature-data"'));}
console.log('Manual models: four storage plans, 16 bitmap states, RU/EN DOM hooks passed.');
