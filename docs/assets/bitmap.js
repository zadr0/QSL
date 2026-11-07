(()=>{'use strict';
function model(a,b,invalid,table){const mask=(a?128:0)|(b?64:0)|(invalid?1:0),indices=[];let remaining=mask,valid=true;
 if(remaining){let trailing=0;while(((remaining>>>trailing)&1)===0)trailing++;valid=(8-trailing)<=2;}
 if(valid)while(remaining){let trailing=0;while(((remaining>>>trailing)&1)===0)trailing++;indices.push(8-trailing);remaining&=remaining-1;}
 return {mask,valid,indices,calls:indices.map(i=>table?'OT['+(i-1)+']':'match '+i),wire:[mask,9,...(b?[7]:[]),...(a?[0x34,0x12]:[])]};}
const $=id=>document.querySelector('#'+id);
function render(){const m=model($('bm-a').checked,$('bm-b').checked,$('bm-invalid').checked,$('bm-ot').checked);
 $('bm-mask').textContent=m.mask.toString(2).padStart(8,'0')+' · 0x'+m.mask.toString(16).padStart(2,'0').toUpperCase();$('bm-order').textContent=m.valid?(m.indices.join(' → ')||'∅'):'—';$('bm-readers').textContent=m.valid?(m.calls.join(' → ')||'∅'):'—';$('bm-safety').textContent=m.valid?'Ok':'Err(InvalidValue)';$('bm-safety').className=m.valid?'valid':'error';
 const wire=$('bm-wire');wire.replaceChildren();m.wire.forEach((v,i)=>{const n=document.createElement('span');n.className=i?'byte':'byte meta';n.textContent=v.toString(16).padStart(2,'0').toUpperCase();wire.append(n)});}
globalThis.QslBitmapDoc={model,render};if($('bm-a')){for(const id of ['bm-a','bm-b','bm-ot','bm-invalid'])$(id).addEventListener('change',render);render();}
})();
