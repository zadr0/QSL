const assert=require('node:assert/strict'),fs=require('node:fs');
const m=require('../docs/assets/workshop.js');
for(const values of [[0,0,0],[7,31,1023],[5,17,513],[1,2,256]])for(const mode of ['raw','compact','aligned']){
 const wire=m.dense(values,mode).bytes;assert.deepEqual(m.decodeDense(wire,mode),values);assert.throws(()=>m.decodeDense(wire.slice(0,-1),mode));
 const bad=wire.slice();bad[mode==='compact'?2:0]|=128;assert.throws(()=>m.decodeDense(bad,mode));
}
assert.throws(()=>m.dense([8,0,0],'raw'));
for(let mask=0;mask<4;mask++){const p=m.optional(mask);assert.equal(p.I,p.L);assert.equal(p.bytes.length,5+p.L);}
const cases=[new Uint8Array(),Uint8Array.of(255),new Uint8Array(128).fill(65),Uint8Array.from({length:256},(_,i)=>i),new TextEncoder().encode('Рад, так Рад. '.repeat(64))];
for(const q of [2,4,8,16])for(const bytes of cases){const r=m.qpsBytes(bytes,q);assert.deepEqual(r.restored,bytes);assert.deepEqual(m.decodeQpsFrame(r.frame),bytes);for(let n=0;n<r.frame.length;n++)assert.throws(()=>m.decodeQpsFrame(r.frame.slice(0,n)));const forged=r.frame.slice();new DataView(forged.buffer).setBigUint64(8,8193n,true);assert.throws(()=>m.decodeQpsFrame(forged));}
for(const lang of ['ru','en']){const page=fs.readFileSync('docs/'+(lang==='ru'?'index.html':'en.html'),'utf8');assert.ok(page.includes('id="full-guide"'));assert.ok(page.includes('id="attribute-catalogue"'));assert.equal((page.match(/<article class="book full-guide">/g)||[]).length,1);assert.ok(!page.includes('data-locale="'+lang+'" hidden'));for(const id of ['workshop','layout-lab','qps-demo','event-demo','strings','optional','measurements','api'])assert.ok(page.includes('id="'+id+'"'));}
console.log('Workshop: layout validation, all PS states, Q2/Q4/Q8/Q16 round trips and malformed lengths/truncations; full visible RU/EN guide passed.');
