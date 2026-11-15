import {readdirSync, readFileSync} from 'node:fs';
import {join} from 'node:path';
import {LOCALE_PREFIXES} from '../config/site.mjs';
const root='src/content/docs';
function files(dir,prefix=''){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?files(join(dir,e.name),prefix+e.name+'/'):/[.]mdx?$/.test(e.name)?[{route:prefix+e.name.replace(/[.]mdx?$/,''),file:join(dir,e.name)}]:[])}
const base=files(join(root,'en'));const expected=new Set(base.map(p=>p.route));let failures=[];
for(const locale of LOCALE_PREFIXES){const pages=files(join(root,locale));const routes=new Set(pages.map(p=>p.route));for(const route of expected)if(!routes.has(route))failures.push(locale+': missing '+route);for(const route of routes)if(!expected.has(route))failures.push(locale+': unexpected '+route);for(const page of pages){const text=readFileSync(page.file,'utf8');if(!/^---[\r\n]/.test(text)||!/^title:/m.test(text)||!/^description:/m.test(text))failures.push(page.file+': missing metadata');if(text.includes('\ufffd'))failures.push(page.file+': invalid Unicode')}}
if(failures.length){console.error(failures.join('\n'));process.exit(1)}console.log(LOCALE_PREFIXES.length+' locales, '+expected.size+' routes each: coverage and metadata passed');
