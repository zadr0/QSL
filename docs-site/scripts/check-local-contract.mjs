import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
const root=path.resolve(import.meta.dirname,'..');const locales=['en','ru','zh-cn','ja','ko','es','pt-br','de','fr'];const docs=path.join(root,'src/content/docs');
const walk=d=>fs.readdirSync(d,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(path.join(d,e.name)):[path.join(d,e.name)]);
const routes=l=>walk(path.join(docs,l)).filter(f=>/\.mdx?$/.test(f)).map(f=>path.relative(path.join(docs,l),f).replaceAll(path.sep,'/').replace(/\.mdx?$/,''));
const expected=routes('en').sort();const checks=[];
for(const locale of locales){
 const actual=routes(locale).sort();assert.equal(new Set(actual).size,actual.length,`duplicate ${locale}`);assert.deepEqual(actual,expected,`route coverage ${locale}`);
 const home=fs.readFileSync(path.join(root,'dist',locale,'index.html'),'utf8');assert.match(home,/id="usage"/);assert.match(home,/0\.3\.0/);assert(!home.includes('<qsl-packing-strip'));assert(!home.includes('class="home-tools'));
 const footer=home.match(/<nav class="qsl-footer__tools[^"]*"[\s\S]*?<\/nav>/)?.[0]||'';assert.equal((footer.match(/<a /g)||[]).length,3,`footer tools ${locale}`);
 for(const route of ['guides/manual-layout','guides/custom-types']){const html=fs.readFileSync(path.join(root,'dist',locale,route,'index.html'),'utf8');assert(!html.includes('katex-error'));if(route.endsWith('manual-layout'))assert(html.includes('class="katex"'),`math ${locale}`);}
 checks.push(`${locale}: ${actual.length} routes, home, footer, formula rendering`);
}
for(const file of walk(docs).filter(f=>/\.mdx?$/.test(f))){if(file.includes(path.sep+'lab'+path.sep))continue;assert(!/<(?:QslLab|SchemaGraph|PackingStrip)\b/.test(fs.readFileSync(file,'utf8')),`interactive component outside labs: ${file}`)}
const server='http://127.0.0.1:4322';
const post=async(endpoint,value)=>{const r=await fetch(server+'/api/qsl/'+endpoint+'/',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(value)});assert.equal(r.status,200);return await r.json()};
assert((await(await fetch(server+'/api/qsl/health/')).json()).ok);
const sample=await post('run',{demo:'numbers',mode:'compact'});assert(sample.ok);assert.equal(sample.hex,'8d 01 02');assert.equal(sample.decoded.value,513);
for(const mode of ['raw','stack','lz']){const r=await post('run',{demo:'strings',mode,text:'a'.repeat(128)});assert(r.ok);assert.equal(r.decoded.text,'a'.repeat(128));if(mode==='lz')assert(r.wireBytes<128);const bad=await post('run',{demo:'strings',mode,hex:r.hex.split(' ').slice(0,-1).join(' ')});assert.equal(bad.ok,false)}
for(const mode of ['default','ps','ot','both','bitmap']){const r=await post('run',{demo:'optional',mode,a:5,b:null,c:100});assert(r.ok);assert.deepEqual(r.decoded,{id:9,a:5,b:null,c:100})}
for(const quantum of [2,4,8,16]){const r=await post('run',{demo:'qps',quantum,text:'Яa'.repeat(80)});assert(r.ok);assert.equal(r.decoded.text,'Яa'.repeat(80))}
for(const mode of ['vec','inline']){const r=await post('run',{demo:'arrays',mode,data:[0,1,65535]});assert(r.ok);assert.deepEqual(r.decoded.data,[0,1,65535])}
const pinned=await post('run',{demo:'layout',mode:'pinned'}),auto=await post('run',{demo:'layout',mode:'auto'});assert(pinned.ok&&auto.ok);assert.equal(pinned.hex,auto.hex);assert(auto.objectBytes<pinned.objectBytes);
const schema=await post('analyze',{source:'&P[safety,auto]{u8 flag;u64 count;u16 id;}'});assert(schema.ok);assert.equal(schema.schemas[0].name,'P');
const invalid=await post('analyze',{source:'&P[safety]{u8? opt;u32 required;}'});assert.equal(invalid.ok,false);
checks.push('Native: number golden, UTF-8 new/storage/LZ, truncation rejection, five optional modes, four QPS quanta, two array modes, physical auto and invalid schema rejection.');
console.log(checks.join('\n'));
