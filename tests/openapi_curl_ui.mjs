import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';

const source = readFileSync(new URL('../src/theme/default/docanvil.js', import.meta.url), 'utf8')
  .split('// Header copy actions enhance commands that remain readable without JavaScript.')[1];
const fixture = () => ({
  prefix: "curl --globoff --request 'POST'", path: '/items/a%2Fb', args: ["--header 'X-Test: yes'"], query: ['limit=20'], cookies: ['session=a%20b'],
  servers: [{label:'Production',base:'https://prod.test/v1?fixed=yes',notes:[]},{label:'Sandbox',base:"https://sand'box.test/v2",notes:['Replace this example origin.']}],
  authentication: [
    {label:'Token + key',args:["--header 'Authorization: Bearer REPLACE_TOKEN'"],query:['key=REPLACE_API_KEY'],cookies:['key=REPLACE_API_KEY'],notes:[]},
    {label:'Basic',args:["--user 'USERNAME:PASSWORD'"],query:[],cookies:[],notes:['<literal> [[notes]]']},
    {label:'Anonymous',args:[],query:[],cookies:[],notes:[]}
  ],
  bodies:[
    {media:'application/json',label:'Draft',args:["--data-raw '{\"state\":\"draft\"}'"],notes:[]},
    {media:'application/json',label:'Published',args:["--data-raw '{\"state\":\"published\"}'"],notes:[]},
    {media:'text/plain',label:'Text',args:["--data-raw '@literal'"],notes:['Literal @ text.']}
  ]
});

function setup({clipboard='success', invalid=false, missing=false}={}) {
  const documentEvents={}, windowEvents={};
  let document;
  class Element {
    constructor(tag='div') { this.tag=tag;this.children=[];this.events={};this.hidden=false;this.textContent='';this.attrs={}; }
    get value() { return this._value ?? (this.tag==='select' ? this.children[0]?.value ?? '' : ''); }
    set value(value) { this._value=String(value); }
    get options() { return this.children; }
    appendChild(child) { this.children.push(child);return child; }
    replaceChildren(...children) { this.children=children;this._value=undefined; }
    addEventListener(name,fn) { this.events[name]=fn; }
    setAttribute(name,value) { this.attrs[name]=value; }
    focus() { document.activeElement=this; }
  }
  const names=['controls','fallback','server','auth','media','example','example-label','copy','show','preview','preview-code','selected-notes'];
  const nodes=Object.fromEntries(names.map(name=>[name,new Element(['server','auth','media','example'].includes(name)?'select':'div')]));
  const summary=new Element(),status=new Element(),dropdown=new Element();dropdown.open=false;
  nodes.controls.hidden=true;nodes.preview.hidden=true;
  for (const name of ['server','auth']) nodes[name].value='0';
  nodes.media.value='application/json';
  nodes.example.value='0';
  dropdown.getAttribute=()=>invalid?'{bad JSON':JSON.stringify(fixture());
  dropdown.querySelector=selector=> {
    if (selector==='summary')return summary;
    if (selector==='.api-curl-status')return status;
    const name=selector.slice('[data-api-curl-'.length,-1);
    return missing && name==='server' ? null : nodes[name];
  };
  dropdown.contains=target=>target===summary||Object.values(nodes).includes(target);
  document={activeElement:null,createElement:tag=>new Element(tag),querySelectorAll:()=>[dropdown],addEventListener:(name,fn)=>documentEvents[name]=fn};
  const window={addEventListener:(name,fn)=>windowEvents[name]=fn};
  let copied, finishCopy;
  const navigator=clipboard==='missing'?{}:{clipboard:{writeText:async text=>{
    copied=text;
    if(clipboard==='failure')throw Error('clipboard denied');
    if(clipboard==='delayed')await new Promise(resolve=>finishCopy=resolve);
  }}};
  const context=vm.createContext({document,window,navigator});
  vm.runInContext(source,context);
  return {nodes,summary,status,dropdown,document,documentEvents,windowEvents,context,getCopied:()=>copied,finishCopy:()=>finishCopy()};
}

test('server/auth/example selections produce one current command and preserve parameter cookies',()=>{
  const s=setup(),n=s.nodes;
  assert.equal(n.controls.hidden,false);assert.equal(n.fallback.hidden,true);
  assert.match(n['preview-code'].textContent,/https:\/\/prod.test\/v1\/items\/a%2Fb\?fixed=yes&limit=20&key=REPLACE_API_KEY/);
  n.server.value='1';n.server.events.change();
  n.auth.value='1';n.auth.events.change();
  n.example.value='1';n.example.events.change();
  const command=n['preview-code'].textContent;
  assert.match(command,/--user 'USERNAME:PASSWORD'/);assert.match(command,/published/);
  assert.ok(command.includes("--cookie 'session=a%20b'"));assert.ok(command.includes("sand'\\''box.test"));
  assert.ok(!command.includes('REPLACE_API_KEY'));assert.ok(!command.includes('Bearer'));
  assert.equal(n['selected-notes'].children[1].textContent,'<literal> [[notes]]');
  n.auth.value='2';n.auth.events.change();
  assert.ok(!n['preview-code'].textContent.includes('--user'));
});

test('media switching resets to a valid example and hides the single-example selector',()=>{
  const {nodes:n}=setup();
  assert.equal(n.example.options.length,2);assert.equal(n['example-label'].hidden,false);
  n.example.value='1';n.example.events.change();
  n.media.value='text/plain';n.media.events.change();
  assert.equal(n.example.value,'2');assert.equal(n.example.options.length,1);assert.equal(n['example-label'].hidden,true);
  assert.ok(n['preview-code'].textContent.includes("--data-raw '@literal'"));
  assert.ok(!n['preview-code'].textContent.includes('published'));
  n.media.value='application/json';n.media.events.change();
  assert.equal(n.example.value,'0');assert.equal(n.example.options.length,2);
});

test('copy uses the selected command, show command toggles and selection clears old success',async()=>{
  const s=setup(),n=s.nodes;
  n.example.value='1';n.example.events.change();
  await n.copy.events.click();assert.equal(s.getCopied(),n['preview-code'].textContent);assert.match(s.status.textContent,/copied/);
  n.show.events.click();assert.equal(n.preview.hidden,false);assert.equal(n.show.attrs['aria-expanded'],'true');
  n.show.events.click();assert.equal(n.preview.hidden,true);
  n.server.value='1';n.server.events.change();assert.equal(s.status.textContent,'');
});

test('clipboard denial reveals the currently selected command for manual copying',async()=>{
  const s=setup({clipboard:'failure'}),n=s.nodes;
  n.media.value='text/plain';n.media.events.change();
  await n.copy.events.click();assert.equal(n.preview.hidden,false);assert.match(s.status.textContent,/Select and copy/);
  assert.ok(n['preview-code'].textContent.includes("--data-raw '@literal'"));
});

test('choices still work without clipboard access',()=>{
  const s=setup({clipboard:'missing'}),n=s.nodes;
  assert.equal(n.copy.hidden,true);assert.equal(n.preview.hidden,false);assert.match(s.status.textContent,/Clipboard unavailable/);
  n.auth.value='2';n.auth.events.change();assert.ok(!n['preview-code'].textContent.includes('Bearer'));
});

test('an earlier asynchronous clipboard result cannot overwrite a newer selection status',async()=>{
  const s=setup({clipboard:'delayed'});
  const pending=s.nodes.copy.events.click();
  s.nodes.server.value='1';s.nodes.server.events.change();s.finishCopy();await pending;
  assert.equal(s.status.textContent,'');
});

test('Escape restores focus, outside clicks dismiss and printing restores open state',()=>{
  const s=setup();s.dropdown.open=true;
  s.dropdown.events.keydown({key:'Escape',preventDefault(){}});assert.equal(s.dropdown.open,false);assert.equal(s.document.activeElement,s.summary);
  s.dropdown.open=true;s.documentEvents.click({target:{}});assert.equal(s.dropdown.open,false);
  s.windowEvents.beforeprint();assert.equal(s.dropdown.open,true);s.windowEvents.afterprint();assert.equal(s.dropdown.open,false);
});

test('invalid or incomplete widget data retains the static fallback',()=>{
  for(const options of [{invalid:true},{missing:true}]){
    const s=setup(options);assert.equal(s.nodes.controls.hidden,true);assert.equal(s.nodes.fallback.hidden,false);
  }
});

test('POSIX quote protects URL apostrophes, substitutions and newlines',{skip:process.platform==='win32'},()=>{
  const s=setup();const raw="don't `echo unsafe` $(echo unsafe)\n'quoted'";
  const quoted=s.context.apiCurlQuote(raw);
  assert.equal(execFileSync('sh',['-c',`printf %s ${quoted}`],{encoding:'utf8'}),raw);
});

test('the assembled multiline command passes literal values to curl',{skip:process.platform==='win32'},()=>{
  const s=setup(),data=fixture();
  const payload="don't $(printf unsafe) `echo unsafe`\n[[literal]]";
  data.bodies[0].args=['--data-raw '+s.context.apiCurlQuote(payload)];
  const command=s.context.apiCurlCommand(data,1,0,0);
  const output=execFileSync('sh',['-c',"curl() { printf '%s\\0' \"$@\"; };\n"+command],{encoding:'utf8'}).split('\0');
  assert.equal(output[output.indexOf('--data-raw')+1],payload);
  assert.equal(output[output.indexOf('--cookie')+1],'session=a%20b; key=REPLACE_API_KEY');
  assert.equal(output[output.indexOf('--url')+1],"https://sand'box.test/v2/items/a%2Fb?limit=20&key=REPLACE_API_KEY");
});
