use serde_json::{json,Value};
use std::{io::{self,Read},mem::size_of};
include!(concat!(env!("OUT_DIR"),"/schema.rs"));
type Res=Result<Value,String>;
fn hex(bytes:&[u8])->String{bytes.iter().map(|b|format!("{b:02x}")).collect::<Vec<_>>().join(" ")}
fn input_wire(v:&Value)->Result<Option<Vec<u8>>,String>{
 let Some(s)=v.get("hex").and_then(Value::as_str) else{return Ok(None)};
 let s=s.split_whitespace().collect::<String>();
 if s.len()>131070||s.len()%2!=0||!s.is_ascii(){return Err("Invalid hex frame".into())}
 (0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).map_err(|_|"Invalid hex frame".to_string())).collect::<Result<Vec<_>,_>>().map(Some)
}
fn num(v:&Value,key:&str,default:u64,max:u64)->Result<u64,String>{
 let n=match v.get(key){None=>default,Some(x)=>x.as_u64().ok_or_else(||format!("{key}: expected unsigned integer"))?};
 if n>max{return Err(format!("{key}: InvalidValue (max {max})"))}Ok(n)
}
fn optional(v:&Value,key:&str,max:u64)->Result<Option<u64>,String>{if v.get(key).is_none_or(Value::is_null){Ok(None)}else{num(v,key,0,max).map(Some)}}
macro_rules! run{($ty:ty,$value:expr,$decode:expr,$useful:expr,$storage:expr,$input:expr)=>{{
 let value=$value;let wire=if let Some(w)=$input{w}else{value.serialize_checked().map_err(|e|format!("{e:?}"))?};
 let back=<$ty>::deserialize_checked(&wire).map_err(|e|format!("{e:?}"))?;
 Ok(json!({"ok":true,"wire":wire,"hex":hex(&wire),"wireBytes":wire.len(),"objectBytes":size_of::<$ty>(),"usefulBytes":$useful,"storage":$storage,"decoded":($decode)(back),"api":"serialize_checked / deserialize_checked","engine":"native QSL","schemaName":stringify!($ty),"schema":include_str!("../schemas.qsl").lines().find(|s|s.starts_with(concat!("&",stringify!($ty),"["))).unwrap_or("")}))
}};}
fn qps<const Q:usize>(v:&Value)->Res{
 let text=v.get("text").and_then(Value::as_str).unwrap_or("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
 let bytes=text.as_bytes();if bytes.len()>4096{return Err("Input limit: 4096 UTF-8 bytes".into())}
 let mut output=vec![0; qsl::qps_bytes::ByteQps::<Q>::max_encoded_len(bytes.len()).map_err(|e|format!("{e:?}"))?];
 let f=qsl::qps_bytes::ByteQps::<Q>::encode_to(bytes,&mut output).map_err(|e|format!("{e:?}"))?;
 let mut restored=vec![0u8;bytes.len()];let result=f.decode_to(&mut restored).map_err(|e|format!("{e:?}"))?;
 Ok(json!({"ok":true,"wire":f.as_slice(),"hex":hex(f.as_slice()),"wireBytes":f.as_slice().len(),"usefulBytes":bytes.len(),"restoredBytes":bytes.len(),"envelopeBytes":16,"mode":format!("{:?}",f.mode()),"decoded":{"text":std::str::from_utf8(result).map_err(|e|e.to_string())?},"api":"ByteQps::encode_to / decode_to","engine":"native QSL"}))
}
fn execute(v:&Value)->Res{
 let demo=v.get("demo").and_then(Value::as_str).unwrap_or("numbers");let mode=v.get("mode").and_then(Value::as_str).unwrap_or("compact");let input=input_wire(v)?;
 match demo{
 "numbers"=>{let kind=num(v,"kind",5,7)? as u8;let channel=num(v,"channel",17,31)? as u8;let value=num(v,"value",513,1023)? as u16;
 macro_rules! sample{($t:ident)=>{run!($t,$t{kind,channel,value},|r:$t|json!({"kind":r.kind,"channel":r.channel,"value":r.value}),4,"u8 + u8 + u16",input)}}
 match mode{"default"=>sample!(SampleRaw),"aligned"=>sample!(SampleAligned),"compact"=>sample!(SampleCompact),_=>Err("Unknown number mode".into())}}
 "strings"=>{let text=v.get("text").and_then(Value::as_str).unwrap_or("Рад, так рад");let tail=num(v,"tail",42,u32::MAX as u64)? as u32;
 if text.len()>255{return Err("InvalidValue: text exceeds 255 UTF-8 bytes".into())}
 match mode{
 "raw"=>run!(TextRaw,TextRaw{text:text.into(),tail},|r:TextRaw|json!({"text":r.text,"tail":r.tail}),text.len()+4,"String",input),
 "stack"=>run!(TextStack,TextStack{text:qsl::StackString::try_from_str(text).map_err(|e|format!("{e:?}"))?,tail},|r:TextStack|json!({"text":r.text.as_str(),"tail":r.tail}),text.len()+4,"StackString<255>",input),
 "lz"=>run!(TextLz,TextLz{text:qsl::InlineString::try_from_str(text).map_err(|e|format!("{e:?}"))?,tail},|r:TextLz|json!({"text":r.text.as_str(),"tail":r.tail}),text.len()+4,"InlineString<255> / QSL-LZ",input),_=>Err("Unknown string mode".into())}}
 "optional"=>{let id=num(v,"id",9,u32::MAX as u64)? as u32;let a=optional(v,"a",255)?.map(|x|x as u8);let b=optional(v,"b",65535)?.map(|x|x as u16);let c=optional(v,"c",u32::MAX as u64)?.map(|x|x as u32);let useful=4+a.is_some() as usize+2*b.is_some() as usize+4*c.is_some() as usize;
 macro_rules! opt{($t:ident)=>{run!($t,$t::try_new(id,a,b,c).map_err(|e|format!("{e:?}"))?,|r:$t|json!({"id":r.id,"a":r.a(),"b":r.b(),"c":r.c()}),useful,"native slots + presence mask",input)}}
 match mode{"default"=>opt!(OptionalDefault),"ps"=>opt!(OptionalPS),"ot"=>opt!(OptionalOT),"both"=>opt!(OptionalBoth),"bitmap"=>opt!(OptionalBM),_=>Err("Unknown optional mode".into())}}
 "layout"=>{let flag=7;let count=1024;let id=42;
 if mode=="auto"{run!(LayoutAuto,LayoutAuto{flag,count,id},|r:LayoutAuto|json!({"flag":r.flag,"count":r.count,"id":r.id}),11,"repr(C), physical auto",input)}else{run!(LayoutPinned,LayoutPinned{flag,count,id},|r:LayoutPinned|json!({"flag":r.flag,"count":r.count,"id":r.id}),11,"repr(C), pinned",input)}}
 "arrays"=>{let data=v.get("data").and_then(Value::as_array).ok_or("data: expected array")?.iter().map(|x|x.as_u64().filter(|x|*x<=65535).map(|x|x as u16).ok_or("Invalid u16".to_string())).collect::<Result<Vec<_>,_>>()?;if data.len()>16{return Err("InvalidValue: more than 16 elements".into())}let useful=data.len()*2;
 if mode=="inline"{run!(ArrayInline,ArrayInline{data:qsl::InlineArray::try_from_slice(&data).map_err(|e|format!("{e:?}"))?},|r:ArrayInline|json!({"data":r.data.as_slice()}),useful,"InlineArray<u16,16>",input)}else{run!(ArrayVec,ArrayVec{data},|r:ArrayVec|json!({"data":r.data}),useful,"Vec<u16>",input)}}
 "qps"=>match num(v,"quantum",16,16)?{2=>qps::<2>(v),4=>qps::<4>(v),8=>qps::<8>(v),16=>qps::<16>(v),_=>Err("Quantum must be Q2/Q4/Q8/Q16".into())},
 _=>Err("Unknown demonstration".into())
 }
}
fn main(){let mut s=String::new();let response=(||{io::stdin().take(65537).read_to_string(&mut s).map_err(|e|e.to_string())?;if s.len()>65536{return Err("Request too large".into())}let v:Value=serde_json::from_str(&s).map_err(|e|e.to_string())?;execute(&v)})();println!("{}",response.unwrap_or_else(|e|json!({"ok":false,"error":e,"engine":"native QSL"})));}
