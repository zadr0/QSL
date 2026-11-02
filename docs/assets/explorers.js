'use strict';
(()=>{
 const $=s=>document.querySelector(s),all=s=>Array.from(document.querySelectorAll(s));
 const words=JSON.parse($('#words').textContent),data=JSON.parse($('#feature-data').textContent);
 const t=k=>words[k]?.[document.documentElement.lang==='ru'?0:1]||k;
 const state={stage:0};
 const node=(tag,text,cls)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=String(text);if(cls)e.className=cls;return e};
 const hex=x=>x.toString(16).padStart(2,'0');
 const fmt=x=>new Intl.NumberFormat(document.documentElement.lang,{maximumFractionDigits:3}).format(x);
 const integer=(x,max)=>{const n=Number(x);if(!Number.isInteger(n)||n<0||n>max)throw Error('domain');return n};
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
 function memoryRow(){return data.rows.find(r=>r.group==='features/strings/'+$('#memory-dataset').value+'/read'&&r.mode===$('#memory-mode').value)}
 function renderMemory(){
  const row=memoryRow(),base=data.rows.find(r=>r.group===row.group&&r.mode==='String_new'),mode=row.mode,borrow=mode.endsWith('_borrow'),reuse=mode.endsWith('_reuse');let action='copy',destination='String',contract='memNew';
  if(mode.startsWith('CopyGuard')){destination='Utf8CopyGuard / SerOut';contract=reuse?'memReuse':'memCopy'}
  else if(mode==='InlineString'){destination='InlineString<4096>';contract='memInline'}
  else if(mode==='string_LZ'){action='raw / LZ';destination='InlineString<4096>';contract='memLz'}
  else if(borrow){action='view';destination=mode==='str_borrow'?"&'a str":mode==='bytes_borrow'?"&'a [u8]":"Utf8Guard<'a>";contract='memBorrow'}
  else if(reuse)contract='memReuse';
  $('#memory-source-size').textContent=fmt(row.wire_mean_bytes)+' B';$('#memory-action').textContent=action;$('#memory-destination').textContent=destination;$('#memory-contract').textContent=t(contract);
  $('#memory-base-time').textContent=fmt(base.ns)+' ns';$('#memory-selected-time').textContent=fmt(row.ns)+' ns';const max=Math.max(base.ns,row.ns);$('#memory-base-bar').style.width=base.ns/max*100+'%';$('#memory-selected-bar').style.width=row.ns/max*100+'%';
  $('#memory-performance').textContent=`M=${fmt(row.useful_bytes)} B · P=${fmt(row.P)} B/ns · CI 99% [${fmt(row.low_ns)}, ${fmt(row.high_ns)}] ns · ${t('snapshot')}: ${row.profile}`;
 }
 function renderEvents(){
  const mask=Number($('#event-a').checked)+2*Number($('#event-b').checked),r=optional(mask);all('[data-event]').forEach(e=>e.setAttribute('data-active',String(Number(e.dataset.event)===r.I)));
  const wire=$('#event-bytes');wire.replaceChildren();r.bytes.forEach((b,i)=>{const e=node('span',hex(b),i===0?'meta':i<5?'required':'optional');e.setAttribute('title',i===0?'H':i<5?'id':mask===2||i===6?'b':'a');wire.append(e)});
 }
 function renderQpsGroup(){
  const bytes=new TextEncoder().encode($('#qps-text').value);if(bytes.length>8192){for(const id of ['qps-ranks','qps-word-bytes','qps-restored-bytes'])$('#'+id).replaceChildren();$('#qps-operand').textContent=t('qpsError');$('#qps-group-value').textContent='—';return}const Q=Number($('#qps-quantum').value),r=globalThis.QslDocsTest.qpsBytes(bytes,Q),slider=$('#qps-group');slider.max=String(Math.max(0,r.blocks.length-1));slider.value=String(Math.min(Number(slider.value)||0,Math.max(0,r.blocks.length-1)));const i=Number(slider.value),block=r.blocks[i];
  $('#qps-group-value').textContent=r.blocks.length?`${i+1} / ${r.blocks.length}`:'0 / 0';const a=$('#qps-ranks'),packed=$('#qps-word-bytes'),restored=$('#qps-restored-bytes');a.replaceChildren();packed.replaceChildren();restored.replaceChildren();
  if(!block){$('#qps-operand').textContent=t('qpsEmpty');$('#qps-raw-note').hidden=true;return}
  const values=bytes.slice(i*Q,i*Q+block.n);values.forEach(v=>{const cell=node('span',undefined,'rank-cell');cell.append(node('small',v),node('strong',v-r.base));a.append(cell)});block.bytes.forEach(v=>packed.append(node('span',hex(v))));const mask=(1n<<BigInt(block.bits))-1n;for(let j=0;j<block.n;j++)restored.append(node('span',Number(block.W>>BigInt(j*block.bits)&mask)+r.base));
  $('#qps-operand').textContent=`a=${r.base} · B=${block.bits} · W=${block.W} · ${block.bytes.length} B`;$('#qps-raw-note').hidden=r.packed;
 }
 function refresh(){renderScope();renderPipeline();renderDense();renderMemory();renderEvents();renderQpsGroup()}
 all('[data-flow]').forEach(b=>b.addEventListener('click',()=>{state.stage=Number(b.dataset.flow);renderPipeline()}));
 for(const id of ['scope-known','scope-agreed','scope-bounded','scope-reuse'])$('#'+id).addEventListener('change',renderScope);
 for(const id of ['lab-kind','lab-channel','lab-value'])$('#'+id).addEventListener('input',renderDense);
 $('#lab-mode').addEventListener('change',renderDense);
 $('#lab-corrupt').addEventListener('change',()=>{if($('#lab-corrupt').checked)$('#lab-truncate').checked=false;renderDense()});
 $('#lab-truncate').addEventListener('change',()=>{if($('#lab-truncate').checked)$('#lab-corrupt').checked=false;renderDense()});
 for(const id of ['memory-dataset','memory-mode'])$('#'+id).addEventListener('change',renderMemory);
 $('#memory-open').addEventListener('click',()=>{const row=memoryRow(),app=globalThis.QslDocsTest;app.state.group=row.group.replace(/\/read$/,'');app.state.op='read';app.state.wave=row.case;app.state.metric='P';$('#operation').value='read';$('#metric').value='P';app.hydrate();$('#measurements').scrollIntoView?.({block:'start'})});
 for(const id of ['event-a','event-b'])$('#'+id).addEventListener('change',renderEvents);
 $('#qps-group').addEventListener('input',renderQpsGroup);$('#qps-text').addEventListener('input',renderQpsGroup);$('#qps-quantum').addEventListener('change',renderQpsGroup);
 all('[data-preset]').forEach(b=>b.addEventListener('click',()=>{$('#qps-text').value=b.dataset.preset==='repeat'?'A'.repeat(128):b.dataset.preset==='range'?'ABCDEFGHIJKLMNOP'.repeat(16):'Рад, так Рад. '.repeat(16);globalThis.QslDocsTest.renderQps();renderQpsGroup()}));
 globalThis.QslExplorer={refresh,dense,decodeDense,optional,state};refresh();
})();
