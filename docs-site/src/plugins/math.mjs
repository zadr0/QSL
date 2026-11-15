import katex from 'katex';
export function latexMath(){
 const render=displayMode=>(node,ctx)=>ctx.replaceNode(node,{raw:katex.renderToString(node.value,{displayMode,throwOnError:true,strict:'error',trust:false,output:'htmlAndMathml'}),mdxExpressions:false});
 return {name:'qsl-latex',math:render(true),inlineMath:render(false)};
}
