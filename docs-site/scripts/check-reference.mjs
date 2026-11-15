import {readdirSync,readFileSync} from 'node:fs';
import {join,relative} from 'node:path';
const root=process.argv[2]??'dist';
function files(dir){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?files(join(dir,e.name)):e.name.endsWith('.html')?[join(dir,e.name)]:[])}
const failures=[];let checked=0;
for(const file of files(root)){
 const html=readFileSync(file,'utf8');const page=relative(root,file);
 if(!/^(en|ru|de|fr|es|pt-br|zh-cn|ja|ko)[\\/]/.test(page))continue;
 if(/http-equiv=["']refresh["']/i.test(html))continue;
 checked++;
 const withoutFooter=html.replace(/<footer\b[^>]*data-project-footer[\s\S]*?<\/footer>/g,'');
 const external=[...withoutFooter.matchAll(/<a\b[^>]*href=["'](https?:\/\/[^"']+)/g)].map(m=>m[1]);
 if(external.length)failures.push(page+': external links above footer: '+external.join(', '));
 if(/qsl-lab|qsl-schema-graph|data-bench-explorer|<textarea\b/.test(html))failures.push(page+': interactive lab rendered');
 if(/(?:^|[\\/])(lab|performance)(?:[\\/]|[.])/.test(page))failures.push(page+': retired public route');
 if(!/data-project-footer/.test(html))failures.push(page+': missing project footer');
}
if(failures.length){console.error(failures.slice(0,30).join('\n'));process.exit(1)}
console.log(checked+' documentation pages: footer-only external links, no labs or measurements');
