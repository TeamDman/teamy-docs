// SPDX-License-Identifier: MPL-2.0
// One layout generates the editable scene and its dependency-free SVG preview.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const out = path.join(root, 'src', 'diagrams');
fs.mkdirSync(out, { recursive: true });
const elements = []; const svg = []; let seq=0;
const esc=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
function base(type,x,y,width,height) { const id=`project-plan-${++seq}`; return {id,type,x,y,width,height,angle:0,strokeColor:'#243047',backgroundColor:'transparent',fillStyle:'solid',strokeWidth:2,strokeStyle:'solid',roughness:0,opacity:100,groupIds:[],frameId:null,roundness:null,seed:seq,version:1,versionNonce:seq,isDeleted:false,boundElements:null,updated:1794528000000,link:null,locked:false}; }
function text(s,x,y,size=18) { const lines=s.split('\n'); elements.push({...base('text',x,y,Math.max(...lines.map(l=>l.length))*size*.56,lines.length*size*1.25),text:s,fontSize:size,fontFamily:2,textAlign:'left',verticalAlign:'top',containerId:null,originalText:s,autoResize:true,lineHeight:1.25}); svg.push(`<text x="${x}" y="${y+size}" font-size="${size}" font-family="Arial,sans-serif" fill="#243047">${lines.map((l,i)=>`<tspan x="${x}" dy="${i?size*1.25:0}">${esc(l)}</tspan>`).join('')}</text>`); }
function box(s,x,y,w=245,h=60,color='#e9f2ff') { elements.push({...base('rectangle',x,y,w,h),backgroundColor:color}); svg.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${color}" stroke="#243047" stroke-width="2"/>`); text(s,x+12,y+10,17); }
function edge(x,y,x2,y2,dashed=false) { const p=[[0,0],[x2-x,y2-y]]; elements.push({...base('arrow',x,y,Math.abs(x2-x),Math.abs(y2-y)),strokeStyle:dashed?'dashed':'solid',points:p,lastCommittedPoint:null,startBinding:null,endBinding:null,startArrowhead:null,endArrowhead:'arrow',elbowed:false}); svg.push(`<path d="M${x},${y} L${x2},${y2}" fill="none" stroke="#243047" stroke-width="2" ${dashed?'stroke-dasharray="7 5"':''} marker-end="url(#arrow)"/>`); }
text('Current project plan',30,20,28);
text('13 November 2026 · conceptual map, not a resolved Cargo graph',30,60,16);
text('Solid: existing dependency/workflow   Dashed: proposed use or prior art',30,87,16);
text('Shared foundations',30,135,22);
box('Facet + formats',30,180); box('Figue / tracing /\ncancellation*',360,180,300); box('Teamy Rust CLI',760,180);
edge(275,210,360,210); edge(660,210,760,210);
box('Vox',360,280,300); box('Teamy MFT\nHuman review',760,280,245,60,'#fff1ce'); edge(150,240,360,310); edge(660,310,760,310);
text('Review gates and existing workflows',30,385,22);
box('SFM cache reuse\nHuman review',30,435,280,65,'#fff1ce');
box('Teamy Broker + mobile\nHuman review',360,435,300,65,'#fff1ce'); box('Teamy Discord',760,435); edge(660,465,760,465);
text('Each reviewed workstream: inspect diff → scoped draft PR → human review → merge',30,525,17);
text('Proposed review tooling',30,580,22);
box('teamy-llm-service / decide\nHuman review',30,630,280,65,'#fff1ce'); box('teamy-review\nProposed, not created',430,630,280,65,'#eef7e8'); edge(310,660,430,660,true);
box('Teamy Rust CLI\nProject foundation',30,730,280); edge(310,760,430,680,true);
box('Makepad Relax + Linkme\nOptional compatibility work',760,630,300,65,'#eef7e8'); edge(760,660,710,660,true);
box('Cloud Terrastodon / TPBAC\nCapability Lab / Teamy Agent',430,730,345); edge(570,730,570,695,true);
text('Proposed capture tooling',30,840,22);
box('Winc / Teamy Windows\nExisting DirectX experiments',30,890,330,65); box('Teamy Studio capture\nPlanning / reuse candidates',520,890,330,65,'#eef7e8'); edge(360,920,520,920,true);
text('Yellow: in-flight human review   Green: proposal or unresolved experiment',30,990,16);
const scene={type:'excalidraw',version:2,source:'https://teamdman.github.io/teamy-docs/',elements,appState:{viewBackgroundColor:'#ffffff',gridSize:null},files:{}};
fs.writeFileSync(path.join(out,'current-project-plan.excalidraw'),JSON.stringify(scene,null,2)+'\n');
fs.writeFileSync(path.join(out,'current-project-plan.svg'),`<svg xmlns="http://www.w3.org/2000/svg" width="1100" height="1040" viewBox="0 0 1100 1040" role="img" aria-labelledby="title desc"><title id="title">Current project dependencies and review gates</title><desc id="desc">Conceptual dependencies, proposed review tooling and capture candidates. Details and evidence are in the accompanying current project plan.</desc><defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto"><path d="M0,0 L0,6 L9,3 z" fill="#243047"/></marker></defs><rect width="1100" height="1040" fill="white"/>${svg.join('\n')}</svg>\n`);
console.log(`Generated ${elements.length} editable elements and SVG from one layout.`);
