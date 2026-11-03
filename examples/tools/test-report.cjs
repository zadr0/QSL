"use strict";
const fs=require("node:fs"),vm=require("node:vm"),assert=require("node:assert/strict");
const html=fs.readFileSync(process.argv[2],"utf8"),payload=JSON.parse(/<script type="application\/json" id="data">([\s\S]*?)<\/script>/.exec(html)[1]),code=/<script>([\s\S]*?)<\/script>/.exec(html)[1];
class Element {
 constructor(tag,attrs={}){this.tag=tag;this.attrs=attrs;this.dataset={};for(const[k,v]of Object.entries(attrs))if(k.startsWith("data-"))this.dataset[k.slice(5)]=v;this.value="";this.textContent="";this.listeners={};this.content="";}
 set innerHTML(value){this.content=value;if(this.tag==="select"){const options=[...value.matchAll(/<option value="([^"]+)"/g)].map(m=>m[1]);if(!options.includes(this.value))this.value=options[0]||"";}}
 get innerHTML(){return this.content;}
 setAttribute(k,v){this.attrs[k]=v;}
 addEventListener(k,fn){this.listeners[k]=fn;}
}
const elements=[],ids=new Map();
for(const match of html.slice(0,html.indexOf('<script type="application/json"')).matchAll(/<([a-z0-9]+)\b([^>]*)>/gi)){
 const attrs=Object.fromEntries([...match[2].matchAll(/([a-z-]+)="([^"]*)"/g)].map(x=>[x[1],x[2]]));
 const el=new Element(match[1],attrs);elements.push(el);if(attrs.id){assert(!ids.has(attrs.id));ids.set(attrs.id,el);}
}
const json=new Element("script");json.textContent=JSON.stringify(payload);ids.set("data",json);
for(const id of ["operation","contract","metric"]){const block=new RegExp(`<select id="${id}">([\\s\\S]*?)<\\/select>`).exec(html);ids.get(id).innerHTML=block[1];}
const context={document:{documentElement:{},getElementById:id=>{assert(ids.has(id),id);return ids.get(id);},querySelectorAll:selector=>elements.filter(e=>selector==="[data-t]"?e.dataset.t:selector==="[data-lang]"?e.dataset.lang:false)},location:{search:""},URLSearchParams,console};
vm.createContext(context);vm.runInContext(code,context);
const check=()=>{for(const id of ["bars","waveform","optional-chart"]){assert(!/NaN|Infinity|undefined/.test(ids.get(id).innerHTML),id);}assert.match(ids.get("selected-case").innerHTML,/<code>/);};
for(const lang of ["ru","en"]){elements.find(e=>e.dataset.lang===lang).listeners.click();assert.equal(context.document.documentElement.lang,lang);check();
 for(const group of Object.keys(payload.groups)){ids.get("group").value=group;ids.get("group").listeners.change();
  for(const op of ["all","read","write","validate"]){ids.get("operation").value=op;
   for(const contract of ["all","checked","trusted"]){ids.get("contract").value=contract;
    for(const metric of ["time","p"]){ids.get("metric").value=metric;ids.get("metric").listeners.change();check();}
   }
  }
 }
}
for(const row of payload.rows){ids.get("wave-case").value=row.case;ids.get("wave-case").listeners.change();assert(ids.get("selected-case").innerHTML.includes(row.case));assert.equal(ids.get("group").value,row.group);check();assert.equal(row.sample_ns.length,200);assert(row.time_low<=row.time_ns&&row.time_ns<=row.time_high);assert(Math.abs(row.p-row.useful_bytes/row.time_ns)<1e-9);}
assert.match(ids.get("waveform").innerHTML,/polyline/);
console.log(`PASS: ${payload.rows.length} measured rows; all groups, contracts, operations, P/T, RU/EN and 200-point waveforms`);
