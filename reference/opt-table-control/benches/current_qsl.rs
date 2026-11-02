//! QSL-only control: default vs opt_table, N=1/2/3/5/10 optional fields.
use criterion::{Criterion, criterion_group, criterion_main};
use qsl::Serializable;
use qsl_opt_table_control::*;
use std::{hint::black_box, mem::MaybeUninit, time::Duration};
#[repr(C, align(64))]
struct Wire {
    bytes: [u8; 128],
    len: usize,
}
#[repr(C, align(64))]
struct Output<T>(MaybeUninit<T>);
macro_rules! pair {
 ($c:expr, $n:literal, $raw:ident, $table:ident, [$($f:ident:$bit:literal),+]) => {{
  // Every pair cycles the same 1024 masks. Smaller domains repeat evenly.
  let raw:Vec<_>=(0usize..1024).map(|i| {let m=(i*405+17)&1023;$raw {id:(i as u32).wrapping_mul(7919),$($f:((m>>$bit)&1!=0).then_some((i as u64).wrapping_mul(0x9e3779b97f4a7c15)^($bit as u64)),)+}}).collect();
  let table:Vec<_>=raw.iter().map(|v|$table {id:v.id,$($f:v.$f,)+}).collect();
  let raws:Vec<_>=raw.iter().map(|v|{let mut f=Wire {bytes:[0;128],len:0};f.len=unsafe{v.serialize_into_unsafe(f.bytes.as_mut_ptr())};$raw::validate(&f.bytes[..f.len]).unwrap();f}).collect();
  let tables:Vec<_>=table.iter().map(|v|{let mut f=Wire {bytes:[0;128],len:0};f.len=unsafe{v.serialize_into_unsafe(f.bytes.as_mut_ptr())};$table::validate(&f.bytes[..f.len]).unwrap();f}).collect();
  // Validate each format independently and compare all logical fields.
  for (a,b) in raws.iter().zip(&tables) {
   let x=$raw::deserialize_checked(&a.bytes[..a.len]).unwrap();let y=$table::deserialize_checked(&b.bytes[..b.len]).unwrap();
   assert_eq!((x.id,$(x.$f,)+),(y.id,$(y.$f,)+));
   let mut x=MaybeUninit::<$raw>::uninit();let mut y=MaybeUninit::<$table>::uninit();
   unsafe {$raw::deserialize_into(a.bytes.as_ptr(),a.len,x.as_mut_ptr());$table::deserialize_into(b.bytes.as_ptr(),b.len,y.as_mut_ptr());let x=x.assume_init();let y=y.assume_init();assert_eq!((x.id,$(x.$f,)+),(y.id,$(y.$f,)+));}
   assert_eq!(a as *const _ as usize%64,0);assert_eq!(b as *const _ as usize%64,0);
  }
  let mut g=$c.benchmark_group(concat!("opt_table_control/deserialize_into/n",stringify!($n)));
  g.bench_function("default",|b|{let mut i=0;let mut out=Output(MaybeUninit::<$raw>::uninit());b.iter(||{let f=black_box(&raws[i]);unsafe{$raw::deserialize_into(f.bytes.as_ptr(),f.len,black_box(out.0.as_mut_ptr()));}black_box(&out);i=(i+1)&1023;});});
  g.bench_function("opt_table",|b|{let mut i=0;let mut out=Output(MaybeUninit::<$table>::uninit());b.iter(||{let f=black_box(&tables[i]);unsafe{$table::deserialize_into(f.bytes.as_ptr(),f.len,black_box(out.0.as_mut_ptr()));}black_box(&out);i=(i+1)&1023;});});
  g.finish();
  let mut g=$c.benchmark_group(concat!("opt_table_control/serialize_into/n",stringify!($n)));
  g.bench_function("default",|b|{let mut i=0;let mut out=Wire {bytes:[0;128],len:0};b.iter(||{let v=black_box(&raw[i]);black_box(unsafe{v.serialize_into_unsafe(black_box(out.bytes.as_mut_ptr()))});black_box(&out);i=(i+1)&1023;});});
  g.bench_function("opt_table",|b|{let mut i=0;let mut out=Wire {bytes:[0;128],len:0};b.iter(||{let v=black_box(&table[i]);black_box(unsafe{v.serialize_into_unsafe(black_box(out.bytes.as_mut_ptr()))});black_box(&out);i=(i+1)&1023;});});
  g.finish();
 }};
}
fn measure(c: &mut Criterion) {
    pair!(c,1,Default1,Table1,[f0:0]);
    pair!(c,2,Default2,Table2,[f0:0,f1:1]);
    pair!(c,3,Default3,Table3,[f0:0,f1:1,f2:2]);
    pair!(c,5,Default5,Table5,[f0:0,f1:1,f2:2,f3:3,f4:4]);
    pair!(c,10,Default10,Table10,[f0:0,f1:1,f2:2,f3:3,f4:4,f5:5,f6:6,f7:7,f8:8,f9:9]);
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(15)).measurement_time(Duration::from_secs(10)).sample_size(200).confidence_level(0.99).significance_level(0.01);targets=measure}
criterion_main!(benches);
