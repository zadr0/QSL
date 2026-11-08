// Concrete format models retained from the existing QSL documentation. Not a runtime speed simulator.
'use strict';
(()=>{
 if(typeof module!=='undefined'&&module.exports)module.exports={dense,decodeDense,optional,qpsBytes,decodeQpsFrame};
 if(typeof document==='undefined')return;
 const $=s=>document.querySelector(s),all=s=>Array.from(document.querySelectorAll(s));
 const words=JSON.parse($('#workshop-words').textContent);
 const t=k=>words[k]?.[document.documentElement.lang==='ru'?0:1]||k;
 const state={stage:0};
 const node=(tag,text,cls)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=String(text);if(cls)e.className=cls;return e};
 const hex=x=>x.toString(16).padStart(2,'0');
 const fmt=x=>new Intl.NumberFormat(document.documentElement.lang,{maximumFractionDigits:3}).format(x);
 function integer(x,max){const n=Number(x);if(!Number.isInteger(n)||n<0||n>max)throw Error('domain');return n;}
 function dense(values,mode){
  if(!['raw','compact','aligned'].includes(mode))throw Error('mode');
  const v=[integer(values[0],7),integer(values[1],31),integer(values[2],1023)];
  const widths=mode==='compact'?[3,5,10]:[8,8,16],domain=[3,5,10],owners=[],offsets=[];let W=0n,offset=0;
  v.forEach((x,i)=>{offsets.push(offset);W|=BigInt(x)<<BigInt(offset);for(let bit=0;bit<widths[i];bit++)owners.push(bit<domain[i]?i:mode==='aligned'?-1:i);offset+=widths[i]});
  const bytes=Uint8Array.from({length:Math.ceil(offset/8)},(_,i)=>Number(W>>BigInt(i*8)&255n));while(owners.length<bytes.length*8)owners.push(-1);
  return {bytes,widths,offsets,owners,values:v};
 }
 function decodeDense(bytes,mode){
  const n=mode==='compact'?3:4;if(bytes.length!==n)throw Error(bytes.length<n?'layoutEof':'layoutRange');
  if(mode==='compact'){if(bytes[2]&252)throw Error('layoutPad');return[bytes[0]&7,bytes[0]>>3,bytes[1]|bytes[2]<<8]}
  const v=[bytes[0],bytes[1],bytes[2]|bytes[3]<<8];if(v[0]>7||v[1]>31||v[2]>1023)throw Error(mode==='aligned'?'layoutPad':'layoutRange');return v;
 }
 function optional(mask){integer(mask,3);const L=Number(!!(mask&1))+Number(!!(mask&2));return{L,I:Number(L>0)+Number(L>1),bytes:Uint8Array.from([mask,42,0,0,0,...(mask&1?[11]:[]),...(mask&2?[22]:[])])}}
 function renderScope(){
  const known=$('#scope-known').checked,agreed=$('#scope-agreed').checked,bounded=$('#scope-bounded').checked,reuse=$('#scope-reuse').checked;
  const rows=[['scopeCode',known?'scopeCodeYes':'scopeCodeNo',known],['scopePeer',agreed?'scopePeerYes':'scopePeerNo',agreed],['scopeMax',bounded?'scopeMaxYes':'scopeMaxNo',bounded],['scopeStorage',reuse?'scopeStorageYes':'scopeStorageNo',reuse]],box=$('#scope-result');box.replaceChildren();
  if(!known){rows[2]=['scopeMax','scopeUnknownPlan',false];rows[3]=['scopeStorage','scopeUnknownPlan',false]}
  rows.forEach(([title,text,active])=>{const row=node('div',undefined,'scope-result '+(active?'available':'conditional'));row.append(node('strong',t(title)),node('p',t(text)));box.append(row)});
 }
 const snippets=[
  '&Sample[safety,compact] {\n    u8(0..7) kind;\n    u8(0..31) channel;\n    u16(0..1023) value;\n}',
  'kind:    bit_length(7)    = 3\nchannel: bit_length(31)   = 5\nvalue:   bit_length(1023) = 10\n\n3 + 5 + 10 = 18 bits → 3 B',
  'struct Sample {\n    kind: u8, channel: u8, value: u16,\n}\n\nW = kind | (channel << 3) | (value << 8)\nvalidate(frame) → decode(frame)',
  'value → selected encoder → frame\nframe → checked API → result\n\nString / reusable storage / borrowed view\ncompressed frame → restoration region'
 ];
 function renderPipeline(){
  all('[data-flow]').forEach(b=>b.setAttribute('aria-pressed',String(Number(b.dataset.flow)===state.stage)));
  $('#pipeline-phase').textContent=t(state.stage===3?'runtimeStage':'buildStage');$('#pipeline-title').textContent=t('pipeTitle'+state.stage);$('#pipeline-text').textContent=t('pipeText'+state.stage);$('#pipeline-code').textContent=snippets[state.stage];
 }
 function renderDense(){
  const v=[$('#lab-kind').value,$('#lab-channel').value,$('#lab-value').value].map(Number),mode=$('#lab-mode').value,model=dense(v,mode);let bytes=model.bytes.slice();
  ['kind','channel','value'].forEach((name,i)=>$('#lab-'+name+'-value').textContent=v[i]);
  if($('#lab-corrupt').checked){if(mode==='compact')bytes[2]|=128;else bytes[0]|=128}
  if($('#lab-truncate').checked)bytes=bytes.slice(0,-1);
  $('#lab-wire-size').textContent=bytes.length+' B';const box=$('#lab-byte-grid');box.replaceChildren();
  bytes.forEach((b,i)=>{const tile=node('div',undefined,'byte-tile');tile.append(node('small','@'+i),node('strong',hex(b)),node('span',b));box.append(tile)});
  const bits=$('#lab-bit-grid');bits.replaceChildren();bytes.forEach((b,i)=>{const group=node('div',undefined,'bit-byte');for(let bit=0;bit<8;bit++){const owner=model.owners[i*8+bit],e=node('span',(b>>bit)&1,owner===-1?'pad':'f'+owner);e.setAttribute('title',owner===-1?t('paddingLabel'):['kind','channel','value'][owner]+' · bit '+(i*8+bit-model.offsets[owner]));group.append(e)}bits.append(group)});
  const fields=$('#lab-field-map');fields.replaceChildren();['kind','channel','value'].forEach((name,i)=>{const row=node('div',undefined,'field-line');row.append(node('span',name,'field-dot f'+i),node('code',`[${model.offsets[i]}..${model.offsets[i]+model.widths[i]-1}] · ${model.widths[i]} bits`));fields.append(row)});
  try{const result=decodeDense(bytes,mode);$('#lab-validation').textContent=t('layoutValid');$('#lab-validation').className='valid';$('#lab-restored').textContent=`kind=${result[0]} · channel=${result[1]} · value=${result[2]}`}
  catch(e){$('#lab-validation').textContent=t(e.message);$('#lab-validation').className='error';$('#lab-restored').textContent='—'}
 }
 function renderEventWire(){
  const mask=Number($('#event-a').checked)+2*Number($('#event-b').checked),r=optional(mask);all('[data-event]').forEach(e=>e.setAttribute('data-active',String(Number(e.dataset.event)===r.I)));
  const wire=$('#event-bytes');wire.replaceChildren();r.bytes.forEach((b,i)=>{const e=node('span',hex(b),i===0?'meta':i<5?'required':'optional');e.setAttribute('title',i===0?'H':i<5?'id':mask===2||i===6?'b':'a');wire.append(e)});
 }
function qpsBytes(input,Q){if(![2,4,8,16].includes(Q))throw Error('Q');const base=input.length?input.reduce((a,b)=>Math.min(a,b),255):0,blocks=[];let packedSize=16;
 for(let start=0;start<input.length;start+=Q){const b=input.slice(start,start+Q),bits=32-Math.clz32(b.reduce((a,x)=>a|(x-base),0));let W=0n;b.forEach((v,i)=>W|=BigInt(v-base)<<BigInt(i*bits));const n=Math.ceil(bits*b.length/8),bytes=Array.from({length:n},(_,i)=>Number((W>>BigInt(i*8))&255n));blocks.push({bits,n:b.length,W,bytes});packedSize+=1+n;}
 const packed=packedSize<input.length,frame=new Uint8Array(packed?packedSize:16+input.length);frame.set([81,80,66,51,Q,Number(packed),packed?base:0,0]);new DataView(frame.buffer).setBigUint64(8,BigInt(input.length),true);let cursor=16;if(packed){blocks.forEach(b=>{frame[cursor++]=b.bits;frame.set(b.bytes,cursor);cursor+=b.bytes.length})}else frame.set(input,16);
 const restored=decodeQpsFrame(frame);if(!restored.every((v,i)=>v===input[i]))throw Error('restore');return{frame,restored,base,blocks,packed,packedSize};}
function decodeQpsFrame(frame){
 if(frame.length<16||frame[0]!==81||frame[1]!==80||frame[2]!==66||frame[3]!==51||![2,4,8,16].includes(frame[4])||frame[7]!==0)throw Error('header');
 const N=new DataView(frame.buffer,frame.byteOffset,frame.byteLength).getBigUint64(8,true);if(N>8192n)throw Error('length');const n=Number(N),Q=frame[4],base=frame[6],out=new Uint8Array(n);
 if(frame[5]===0){if(base!==0||frame.length!==16+n)throw Error('raw');out.set(frame.slice(16));return out}if(frame[5]!==1||frame.length>=n)throw Error('packed');let cursor=16;
 for(let start=0;start<n;start+=Q){const count=Math.min(Q,n-start),bits=frame[cursor++];if(bits>8)throw Error('width');const bytes=Math.ceil(count*bits/8);if(cursor+bytes>frame.length)throw Error('eof');let W=0n;for(let i=0;i<bytes;i++)W|=BigInt(frame[cursor++])<<BigInt(i*8);if((W>>BigInt(count*bits))!==0n)throw Error('padding');const mask=(1n<<BigInt(bits))-1n;for(let i=0;i<count;i++){const value=base+Number((W>>BigInt(i*bits))&mask);if(value>255)throw Error('range');out[start+i]=value}}
 if(cursor!==frame.length)throw Error('end');return out;
}

function renderQps(){try{const bytes=new TextEncoder().encode($('#qps-text').value);if(bytes.length>8192)throw Error('length');const result=qpsBytes(bytes,Number($('#qps-quantum').value));$('#qps-source').textContent=bytes.length+' B';$('#qps-wire').textContent=result.frame.length+' B';$('#qps-mode').textContent=result.packed?'packed':'raw';$('#qps-original').textContent=bytes.length+' B';$('#qps-hex').textContent=Array.from(result.frame.slice(0,128),x=>x.toString(16).padStart(2,'0')).join(' ')+(result.frame.length>128?' …':'');$('#qps-detail').textContent=`${t('base')} a=${result.base}; ${t('groups')}=${result.blocks.length}\nQPB3: 16 B; ${t('originalLength')}=${bytes.length} B\n${t('candidate')}=${result.packedSize} B; ${t('restored')} ✓\n${result.blocks.slice(0,5).map((b,i)=>`#${i}: B=${b.bits}; W=${b.W}`).join('\n')}`;$('#qps-error').textContent='';}catch{$('#qps-error').textContent=t('qpsError')}}
function renderEvents(){const a=Number($('#event-a').checked),b=Number($('#event-b').checked),L=a+b,I=Number(L>0)+Number(L>1);$('#event-result').textContent=`L=${L}; I=(L>0)+(L>1)=${I}\n${t('tagRead')}: ${t(I===1?'yes':'no')}\nH=1 B; ${t('frame')}=${5+L} B`;$('#event-forced').textContent=I===0?'∅':I===2?'a, b':'∅';$('#event-possible').textContent=I===0?'∅':'a, b';renderEventWire();}

 function renderQpsGroup(){
  const bytes=new TextEncoder().encode($('#qps-text').value);if(bytes.length>8192){for(const id of ['qps-ranks','qps-word-bytes','qps-restored-bytes'])$('#'+id).replaceChildren();$('#qps-operand').textContent=t('qpsError');$('#qps-group-value').textContent='—';return}const Q=Number($('#qps-quantum').value),r=qpsBytes(bytes,Q),slider=$('#qps-group');slider.max=String(Math.max(0,r.blocks.length-1));slider.value=String(Math.min(Number(slider.value)||0,Math.max(0,r.blocks.length-1)));const i=Number(slider.value),block=r.blocks[i];
  $('#qps-group-value').textContent=r.blocks.length?`${i+1} / ${r.blocks.length}`:'0 / 0';const a=$('#qps-ranks'),packed=$('#qps-word-bytes'),restored=$('#qps-restored-bytes');a.replaceChildren();packed.replaceChildren();restored.replaceChildren();
  if(!block){$('#qps-operand').textContent=t('qpsEmpty');$('#qps-raw-note').hidden=true;return}
  const values=bytes.slice(i*Q,i*Q+block.n);values.forEach(v=>{const cell=node('span',undefined,'rank-cell');cell.append(node('small',v),node('strong',v-r.base));a.append(cell)});block.bytes.forEach(v=>packed.append(node('span',hex(v))));const mask=(1n<<BigInt(block.bits))-1n;for(let j=0;j<block.n;j++)restored.append(node('span',Number(block.W>>BigInt(j*block.bits)&mask)+r.base));
  $('#qps-operand').textContent=`a=${r.base} · B=${block.bits} · W=${block.W} · ${block.bytes.length} B`;$('#qps-raw-note').hidden=r.packed;
 }
 function refresh(){renderScope();renderPipeline();renderDense();renderEvents();renderQps();renderQpsGroup()}
 all('[data-flow]').forEach(b=>b.addEventListener('click',()=>{state.stage=Number(b.dataset.flow);renderPipeline()}));
 for(const id of ['scope-known','scope-agreed','scope-bounded','scope-reuse'])$('#'+id).addEventListener('change',renderScope);
 for(const id of ['lab-kind','lab-channel','lab-value'])$('#'+id).addEventListener('input',renderDense);
 $('#lab-mode').addEventListener('change',renderDense);
 $('#lab-corrupt').addEventListener('change',()=>{if($('#lab-corrupt').checked)$('#lab-truncate').checked=false;renderDense()});
 $('#lab-truncate').addEventListener('change',()=>{if($('#lab-truncate').checked)$('#lab-corrupt').checked=false;renderDense()});
 for(const id of ['event-a','event-b'])$('#'+id).addEventListener('change',renderEvents);
 $('#qps-group').addEventListener('input',renderQpsGroup);$('#qps-text').addEventListener('input',()=>{renderQps();renderQpsGroup()});$('#qps-quantum').addEventListener('change',()=>{renderQps();renderQpsGroup()});
 all('[data-preset]').forEach(b=>b.addEventListener('click',()=>{$('#qps-text').value=b.dataset.preset==='repeat'?'A'.repeat(128):b.dataset.preset==='range'?'ABCDEFGHIJKLMNOP'.repeat(16):'Рад, так Рад. '.repeat(16);renderQps();renderQpsGroup()}));
 globalThis.QslWorkshop={refresh,dense,decodeDense,optional,state};refresh();
})();
