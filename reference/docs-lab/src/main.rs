use qsl::Serializable;
include!(concat!(env!("OUT_DIR"),"/qsl_gen.rs"));
fn main(){
 let mut raw=DenseRaw{kind:5,channel:17,value:513}.serialize();raw[0]|=128;
 let mut compact=DenseCompact{kind:5,channel:17,value:513}.serialize();compact[2]|=128;
 let mut aligned=DenseAligned{kind:5,channel:17,value:513}.serialize();aligned[0]|=128;
 println!("{{\"kind\":\"invalid\",\"mode\":\"raw\",\"error\":\"{:?}\"}}",DenseRaw::validate(&raw).unwrap_err());
 println!("{{\"kind\":\"invalid\",\"mode\":\"compact\",\"error\":\"{:?}\"}}",DenseCompact::validate(&compact).unwrap_err());
 println!("{{\"kind\":\"invalid\",\"mode\":\"aligned\",\"error\":\"{:?}\"}}",DenseAligned::validate(&aligned).unwrap_err());

 for i in 0..32u16 {let k=(i%8)as u8;let c=((i*7)%32)as u8;let v=if i==31{1023}else{(i*37)%1024};
 let raw=DenseRaw{kind:k,channel:c,value:v}.serialize();let packed=DenseCompact{kind:k,channel:c,value:v}.serialize();let aligned=DenseAligned{kind:k,channel:c,value:v}.serialize();
 assert!(DenseRaw::validate(&raw).is_ok());assert!(DenseCompact::validate(&packed).is_ok());assert!(DenseAligned::validate(&aligned).is_ok());
 println!("{{\"kind\":\"dense\",\"values\":[{k},{c},{v}],\"raw\":{raw:?},\"compact\":{packed:?},\"aligned\":{aligned:?}}}");
 }
 for mask in 0..4 {let a=if mask&1!=0{Some(11)}else{None};let b=if mask&2!=0{Some(22)}else{None};let wire=OptionalTwo{id:42,a,b}.serialize();let x=OptionalTwo::deserialize_checked(&wire).unwrap();assert_eq!((x.id,x.a,x.b),(42,a,b));println!("{{\"kind\":\"optional\",\"mask\":{mask},\"wire\":{wire:?}}}");}
}
