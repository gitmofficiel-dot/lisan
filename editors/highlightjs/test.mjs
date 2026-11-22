import { createRequire } from 'module'; const require=createRequire('/tmp/hl/'); const hljs=require('highlight.js/lib/core');
import lisan from './lisan.js';
hljs.registerLanguage('lisan', lisan);
import fs from 'fs';
let ok=true;
for (const f of ['hello','functions','oop_car','errors_and_match','loops','lists']){
  const code=fs.readFileSync(`/tmp/lisan/examples/${f}.lsn`,'utf8');
  const r=hljs.highlight(code,{language:'lisan'});
  const kw=(r.value.match(/hljs-keyword/g)||[]).length, st=(r.value.match(/hljs-string/g)||[]).length, cm=(r.value.match(/hljs-comment/g)||[]).length, fn=(r.value.match(/hljs-title/g)||[]).length;
  console.log(f,{kw,st,cm,fn,relevance:r.relevance, illegal:r.illegal||false}); if(!kw) ok=false;
}
console.log(ok?'PASS':'FAIL');
