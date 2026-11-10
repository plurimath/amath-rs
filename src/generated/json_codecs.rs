// Generated model-specific JSON codecs. Schema declarations are compiled out.
#![allow(unused_mut)]
use super::models::*;
use super::contracts::{ContractError, ContractResult, ErrorReason};
use serde_json::{Value, Map};
fn failure(reason: ErrorReason, id: &str, member: &str, location: &str, detail: impl ToString)->Box<ContractError>{
 Box::new(ContractError{reason,model_id:Some(id.into()),attribute:(!member.is_empty()).then(||member.into()),format:Some("json".into()),rule:None,location:location.into(),klass:None,detail:Some(detail.to_string())})
}
fn object(value:Value,id:&str)->ContractResult<Map<String,Value>> { value.as_object().cloned().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/input","expected object")) }
#[allow(dead_code)]
fn empty(v:&Value)->bool { match v { Value::String(v)=>v.is_empty(),Value::Array(v)=>v.is_empty(),Value::Object(v)=>v.is_empty(),_=>false } }
fn numeric_json_err()->&'static str{"numeric JSON is not a portable exact boundary: the portable-exact-json-v1 profile requires exact values as strings; this looks like ruby-native-v1 numeric JSON (Ruby to_json), which is not accepted"}
// Only a plain 64-bit integer (ordered-content indexes) may be a JSON number; any
// fractional, exponent, negative-zero or oversized spelling is not a portable exact boundary.
fn plain_integer(n:&str)->Result<Value,&'static str>{
 let digits=n.strip_prefix('-').unwrap_or(n);
 let ok=!digits.is_empty()&&digits.bytes().all(|c|c.is_ascii_digit())&&n!="-0"&&(n.parse::<i64>().is_ok()||n.parse::<u64>().is_ok());
 if ok{serde_json::from_str::<Value>(n).map_err(|_|numeric_json_err())}else{Err(numeric_json_err())}
}
// RFC 8259 number grammar, nothing else (no sign prefix, spaces, quotes or leading zeros).
#[allow(dead_code)]
fn json_number(s:&str)->bool{
 let b=s.as_bytes();let mut i=usize::from(b.first()==Some(&b'-'));
 let digits=|i:&mut usize|{let at=*i;while *i<b.len()&&b[*i].is_ascii_digit(){*i+=1;}*i-at};
 let whole=i;let n=digits(&mut i);
 if n==0||(n>1&&b[whole]==b'0'){return false;}
 if i<b.len()&&b[i]==b'.'{i+=1;if digits(&mut i)==0{return false;}}
 if i<b.len()&&(b[i]==b'e'||b[i]==b'E'){i+=1;
  if i<b.len()&&(b[i]==b'+'||b[i]==b'-'){i+=1;}
  if digits(&mut i)==0{return false;}}
 i==b.len()
}
// Numbers in a keyed-collection body that no member mapping extracts would be dropped
// unseen; refuse them like every other loose number (`keep` lists the extracted paths).
#[allow(dead_code)]
fn no_loose_except(v:&Value,keep:&[&[&str]],id:&str)->ContractResult<()>{
 if keep.iter().any(|p|p.is_empty()){return Ok(());}
 match v {
  Value::Object(o)=>o.iter().try_for_each(|(k,x)|{let sub:Vec<&[&str]>=keep.iter().filter(|p|p[0]==k.as_str()).map(|p|&p[1..]).collect();no_loose_except(x,&sub,id)}),
  other=>no_loose(other,id),
 }
}
// Bundles with a `hash` attribute parse numbers loosely; every other position must
// still refuse them, so the decoded model is walked once (`strict_*`).
#[allow(dead_code)]
fn no_loose(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>plain_integer(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e)),
  Value::Array(a)=>a.iter().try_for_each(|x|no_loose(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|no_loose(x,id)),
  _=>Ok(()),
 }
}
// Inside a hash a number must be exact (the same rule the TS strict reader applies).
#[allow(dead_code)]
fn neutral_numbers(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>NeutralNumber::parse(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",format!("{e}: {n}"))),
  Value::Array(a)=>a.iter().try_for_each(|x|neutral_numbers(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|neutral_numbers(x,id)),
  _=>Ok(()),
 }
}
#[allow(dead_code)]
fn strict_each(v:&Value,f:&dyn std::ops::Fn(&Value)->ContractResult<()>)->ContractResult<()>{ match v { Value::Array(a)=>a.iter().try_for_each(f), other=>f(other) } }
// A hash keeps its key order only when serde_json keeps insertion order.
#[allow(dead_code)]
fn ordered_maps(id:&str)->ContractResult<()>{
 let mut m=Map::new(); m.insert("b".into(),Value::Null); m.insert("a".into(),Value::Null);
 if m.keys().next().map(String::as_str)==Some("b"){Ok(())}else{Err(failure(ErrorReason::InvalidDocument,id,"","/input","hash attributes need serde_json with the preserve_order feature"))}
}
fn parse_json(input:&str,id:&str)->ContractResult<Value>{
 // Reject duplicate keys before mapping aliases. serde_json::Value alone would
 // silently keep only the last key, erasing an observable input ambiguity.
 struct Unique(Value);
 impl<'de> serde::Deserialize<'de> for Unique {
  fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
   struct Visitor;
   impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value=Unique;
    fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("JSON without duplicate keys")}
    fn visit_bool<E:serde::de::Error>(self,v:bool)->Result<Unique,E>{Ok(Unique(Value::Bool(v)))}
    // serde_json reads "-0" as the integer 0, so a signed zero is -0 spelled as an integer; keep it as a float
    // that no integer, string or hash position accepts (negative zero is not exact).
    fn visit_i64<E:serde::de::Error>(self,v:i64)->Result<Unique,E>{if NEUTRAL_NUMBERS&&v==0{return Ok(Unique(serde_json::Number::from_f64(-0.0).map_or(Value::Null,Value::Number)));}Ok(Unique(v.into()))}
    fn visit_u64<E:serde::de::Error>(self,v:u64)->Result<Unique,E>{Ok(Unique(v.into()))}
    fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<Unique,E>{Err(E::custom(numeric_json_err()))}
    fn visit_str<E:serde::de::Error>(self,v:&str)->Result<Unique,E>{Ok(Unique(Value::String(v.into())))}
    fn visit_string<E:serde::de::Error>(self,v:String)->Result<Unique,E>{Ok(Unique(Value::String(v)))}
    fn visit_unit<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_none<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut v=Vec::new();while let Some(Unique(x))=a.next_element()?{v.push(x);}Ok(Unique(Value::Array(v)))}
    fn visit_map<A:serde::de::MapAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{
     let mut v=Map::new();let mut first=true;
     while let Some(k)=a.next_key::<String>()?{
      // With serde_json arbitrary_precision (always on for consumers of this crate)
      // every JSON number arrives as a one-entry map, never through visit_f64.
      // The marker's value must be a lone string in the strict JSON number grammar: nothing
      // else may travel through it (no quoted text, objects, spaces or sibling keys).
      if first && k=="$serde_json::private::Number"{let n:String=a.next_value()?;if a.next_key::<String>()?.is_some(){return Err(serde::de::Error::custom("not a JSON number"));}
      if NEUTRAL_NUMBERS{if !json_number(&n){return Err(serde::de::Error::custom("not a JSON number"));}return serde_json::from_str::<Value>(&n).map(Unique).map_err(serde::de::Error::custom);}
      return plain_integer(&n).map(Unique).map_err(serde::de::Error::custom);}
      first=false;
      let Unique(x)=a.next_value()?;
      if v.insert(k,x).is_some(){return Err(serde::de::Error::custom("duplicate JSON key"));}
     }
     Ok(Unique(Value::Object(v)))
    }
   }
   d.deserialize_any(Visitor)
  }
 }
 serde_json::from_str::<Unique>(input).map(|v|v.0).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e))
}
#[allow(dead_code)]
fn get_path(v:&Value,path:&[&str])->Option<Value>{let mut at=v;for part in path{at=at.as_object()?.get(*part)?;}Some(at.clone())}
#[allow(dead_code)]
fn put_path(v:&mut Value,path:&[&str],value:Value,id:&str)->ContractResult<()>{
 if path.is_empty(){if !v.is_null(){return Err(failure(ErrorReason::UnsupportedMapping,id,"","/output","overlapping value mappings"));}*v=value;return Ok(());}
 if v.is_null(){*v=Value::Object(Map::new());}
 let map=v.as_object_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/output","overlapping mapping paths"))?;
 let slot=map.entry(path[0]).or_insert(Value::Null);put_path(slot,&path[1..],value,id)
}
const NEUTRAL_NUMBERS: bool = false;
#[allow(non_snake_case)]
pub fn read_Amath_Model_AmathChainItem_json(input:&str)->ContractResult<Amath_Model_AmathChainItem>{let data=decode_Amath_Model_AmathChainItem(parse_json(input,"Amath::Model::AmathChainItem@default")?)?; let model:Amath_Model_AmathChainItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_AmathChainItem_json(model:&Amath_Model_AmathChainItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_AmathChainItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_AmathChainItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::AmathChainItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("amath_end") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "amath_end", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("amath_end".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("amath_lr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "amath_lr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("amath_lr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("amath_step") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "amath_step", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("amath_step".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathChainItem@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_AmathChainItem(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_AmathChainItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("amath_end"){
if !v.is_null() {
v=decode_Amath_Model_AmathEnd(v)?;
}
data.insert("amath_end".into(),v);}
if let Some(mut v)=data.remove("amath_lr"){
if !v.is_null() {
v=decode_Amath_Model_AmathLr(v)?;
}
data.insert("amath_lr".into(),v);}
if let Some(mut v)=data.remove("amath_step"){
if !v.is_null() {
v=decode_Amath_Model_AmathStep(v)?;
}
data.insert("amath_step".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_AmathChainItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::AmathChainItem@default")?;let mut wire=Map::new();
for mut v in data.remove("amath_end").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_AmathEnd(v)?;
}
wire.insert("amath_end".into(),v);}
for mut v in data.remove("amath_lr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_AmathLr(v)?;
}
wire.insert("amath_lr".into(),v);}
for mut v in data.remove("amath_step").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_AmathStep(v)?;
}
wire.insert("amath_step".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_AmathEnd_json(input:&str)->ContractResult<Amath_Model_AmathEnd>{let data=decode_Amath_Model_AmathEnd(parse_json(input,"Amath::Model::AmathEnd@default")?)?; let model:Amath_Model_AmathEnd=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_AmathEnd_json(model:&Amath_Model_AmathEnd)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_AmathEnd(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_AmathEnd(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::AmathEnd@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathEnd@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_AmathEnd(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_AmathEnd(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_AmathEnd(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::AmathEnd@default")?;let mut wire=Map::new();
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_AmathLr_json(input:&str)->ContractResult<Amath_Model_AmathLr>{let data=decode_Amath_Model_AmathLr(parse_json(input,"Amath::Model::AmathLr@default")?)?; let model:Amath_Model_AmathLr=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_AmathLr_json(model:&Amath_Model_AmathLr)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_AmathLr(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_AmathLr(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::AmathLr@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathLr@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_AmathLr(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_AmathLr(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_AmathLr(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::AmathLr@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_AmathStep_json(input:&str)->ContractResult<Amath_Model_AmathStep>{let data=decode_Amath_Model_AmathStep(parse_json(input,"Amath::Model::AmathStep@default")?)?; let model:Amath_Model_AmathStep=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_AmathStep_json(model:&Amath_Model_AmathStep)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_AmathStep(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_AmathStep(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::AmathStep@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal_2") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "literal_2", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal_2".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::AmathStep@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_AmathStep(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_AmathStep(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("literal_2"){
data.insert("literal_2".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_AmathStep(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::AmathStep@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("literal_2").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal_2".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Base_json(input:&str)->ContractResult<Amath_Model_Base>{let data=decode_Amath_Model_Base(parse_json(input,"Amath::Model::Base@default")?)?; let model:Amath_Model_Base=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Base@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Base_json(model:&Amath_Model_Base)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Base@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Base(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Base@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Base(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Base@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Base@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Base@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Base(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Base(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Base(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Base@default")?;let mut wire=Map::new();
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_BaseValue_json(input:&str)->ContractResult<Amath_Model_BaseValue>{let data=decode_Amath_Model_BaseValue(parse_json(input,"Amath::Model::BaseValue@default")?)?; let model:Amath_Model_BaseValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_BaseValue_json(model:&Amath_Model_BaseValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_BaseValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_BaseValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::BaseValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_BaseValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_BaseValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_BaseValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::BaseValue@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::BaseValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Color_json(input:&str)->ContractResult<Amath_Model_Color>{let data=decode_Amath_Model_Color(parse_json(input,"Amath::Model::Color@default")?)?; let model:Amath_Model_Color=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Color_json(model:&Amath_Model_Color)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Color(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Color(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Color@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Color@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Color(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Color(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Color(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Color@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_ColorValue_json(input:&str)->ContractResult<Amath_Model_ColorValue>{let data=decode_Amath_Model_ColorValue(parse_json(input,"Amath::Model::ColorValue@default")?)?; let model:Amath_Model_ColorValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_ColorValue_json(model:&Amath_Model_ColorValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_ColorValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_ColorValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::ColorValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_ColorValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_ColorValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_ColorValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::ColorValue@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::ColorValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_CommaSeparatedItem_json(input:&str)->ContractResult<Amath_Model_CommaSeparatedItem>{let data=decode_Amath_Model_CommaSeparatedItem(parse_json(input,"Amath::Model::CommaSeparatedItem@default")?)?; let model:Amath_Model_CommaSeparatedItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_CommaSeparatedItem_json(model:&Amath_Model_CommaSeparatedItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_CommaSeparatedItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_CommaSeparatedItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::CommaSeparatedItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::CommaSeparatedItem@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_CommaSeparatedItem(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_CommaSeparatedItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_CommaSeparatedItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::CommaSeparatedItem@default")?;let mut wire=Map::new();
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Denominator_json(input:&str)->ContractResult<Amath_Model_Denominator>{let data=decode_Amath_Model_Denominator(parse_json(input,"Amath::Model::Denominator@default")?)?; let model:Amath_Model_Denominator=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Denominator_json(model:&Amath_Model_Denominator)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Denominator(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Denominator(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Denominator@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Denominator@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Denominator(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Denominator(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Denominator(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Denominator@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Diff_json(input:&str)->ContractResult<Amath_Model_Diff>{let data=decode_Amath_Model_Diff(parse_json(input,"Amath::Model::Diff@default")?)?; let model:Amath_Model_Diff=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Diff@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Diff_json(model:&Amath_Model_Diff)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Diff@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Diff(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Diff@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Diff(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Diff@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Diff@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Diff@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Diff(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Diff(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Diff(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Diff@default")?;let mut wire=Map::new();
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Dividend_json(input:&str)->ContractResult<Amath_Model_Dividend>{let data=decode_Amath_Model_Dividend(parse_json(input,"Amath::Model::Dividend@default")?)?; let model:Amath_Model_Dividend=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Dividend_json(model:&Amath_Model_Dividend)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Dividend(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Dividend(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Dividend@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Dividend(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Dividend(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Dividend(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Dividend@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Dividend@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Divisor_json(input:&str)->ContractResult<Amath_Model_Divisor>{let data=decode_Amath_Model_Divisor(parse_json(input,"Amath::Model::Divisor@default")?)?; let model:Amath_Model_Divisor=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Divisor_json(model:&Amath_Model_Divisor)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Divisor(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Divisor(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Divisor@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("negative_mod_operand") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "negative_mod_operand", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("negative_mod_operand".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Divisor@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Divisor(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Divisor(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("negative_mod_operand"){
if !v.is_null() {
v=decode_Amath_Model_NegativeModOperand(v)?;
}
data.insert("negative_mod_operand".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Divisor(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Divisor@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("negative_mod_operand").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_NegativeModOperand(v)?;
}
wire.insert("negative_mod_operand".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Document_json(input:&str)->ContractResult<Amath_Model_Document>{let data=decode_Amath_Model_Document(parse_json(input,"Amath::Model::Document@default")?)?; let model:Amath_Model_Document=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Document_json(model:&Amath_Model_Document)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Document(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Document(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Document@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("decimal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "decimal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Document@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Document(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Document(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Amath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("decimal"){
data.insert("decimal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Document(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Document@default")?;let mut wire=Map::new();
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("decimal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("decimal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Expr_json(input:&str)->ContractResult<Amath_Model_Expr>{let data=decode_Amath_Model_Expr(parse_json(input,"Amath::Model::Expr@default")?)?; let model:Amath_Model_Expr=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Expr_json(model:&Amath_Model_Expr)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Expr(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Expr(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Expr@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("amath_chain") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "amath_chain", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("amath_chain".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal_2") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "literal_2", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal_2".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_row") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "table_row", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_row".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Expr(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Expr(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("amath_chain"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "amath_chain", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_AmathChainItem(v)?;
*slot=v; }
}
data.insert("amath_chain".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("list"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_ListItem(v)?;
*slot=v; }
}
data.insert("list".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("literal_2"){
data.insert("literal_2".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("table_row"){
if !v.is_null() {
v=decode_Amath_Model_TableRow(v)?;
}
data.insert("table_row".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Expr(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Expr@default")?;let mut wire=Map::new();
for mut v in data.remove("amath_chain").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "amath_chain", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_AmathChainItem(v)?;
*slot=v; }
}
wire.insert("amath_chain".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Expr@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_ListItem(v)?;
*slot=v; }
}
wire.insert("list".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("literal_2").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal_2".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("table_row").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRow(v)?;
}
wire.insert("table_row".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Expression_json(input:&str)->ContractResult<Amath_Model_Expression>{let data=decode_Amath_Model_Expression(parse_json(input,"Amath::Model::Expression@default")?)?; let model:Amath_Model_Expression=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Expression_json(model:&Amath_Model_Expression)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Expression(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Expression(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Expression@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Expression@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Expression(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Expression(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Expression(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Expression@default")?;let mut wire=Map::new();
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_FontsValue_json(input:&str)->ContractResult<Amath_Model_FontsValue>{let data=decode_Amath_Model_FontsValue(parse_json(input,"Amath::Model::FontsValue@default")?)?; let model:Amath_Model_FontsValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_FontsValue_json(model:&Amath_Model_FontsValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_FontsValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_FontsValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::FontsValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_FontsValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_FontsValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_FontsValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::FontsValue@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::FontsValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Frac_json(input:&str)->ContractResult<Amath_Model_Frac>{let data=decode_Amath_Model_Frac(parse_json(input,"Amath::Model::Frac@default")?)?; let model:Amath_Model_Frac=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Frac_json(model:&Amath_Model_Frac)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Frac(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Frac(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Frac@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("denominator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "denominator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("denominator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numerator".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Frac@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Frac(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Frac(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("denominator"){
if !v.is_null() {
v=decode_Amath_Model_Denominator(v)?;
}
data.insert("denominator".into(),v);}
if let Some(mut v)=data.remove("numerator"){
if !v.is_null() {
v=decode_Amath_Model_Numerator(v)?;
}
data.insert("numerator".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Frac(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Frac@default")?;let mut wire=Map::new();
for mut v in data.remove("denominator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Denominator(v)?;
}
wire.insert("denominator".into(),v);}
for mut v in data.remove("numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Numerator(v)?;
}
wire.insert("numerator".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_IntermediateExp_json(input:&str)->ContractResult<Amath_Model_IntermediateExp>{let data=decode_Amath_Model_IntermediateExp(parse_json(input,"Amath::Model::IntermediateExp@default")?)?; let model:Amath_Model_IntermediateExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_IntermediateExp_json(model:&Amath_Model_IntermediateExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_IntermediateExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_IntermediateExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::IntermediateExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("d") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "d", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("d".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("lparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "lparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("lparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rgb_color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "rgb_color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rgb_color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("x") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "x", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("x".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::IntermediateExp@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_IntermediateExp(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_IntermediateExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("d"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("d".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("lparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("lparen".into(),v);}
if let Some(mut v)=data.remove("rgb_color"){
if !v.is_null() {
v=decode_Amath_Model_RgbColor(v)?;
}
data.insert("rgb_color".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("x"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("x".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_IntermediateExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::IntermediateExp@default")?;let mut wire=Map::new();
for mut v in data.remove("d").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("d".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("lparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("lparen".into(),v);}
for mut v in data.remove("rgb_color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_RgbColor(v)?;
}
wire.insert("rgb_color".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("x").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("x".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_LeftRight_json(input:&str)->ContractResult<Amath_Model_LeftRight>{let data=decode_Amath_Model_LeftRight(parse_json(input,"Amath::Model::LeftRight@default")?)?; let model:Amath_Model_LeftRight=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_LeftRight_json(model:&Amath_Model_LeftRight)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_LeftRight(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_LeftRight(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::LeftRight@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("amath_chain") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "amath_chain", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("amath_chain".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "left_right_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_LeftRight(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_LeftRight(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("amath_chain"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "amath_chain", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_AmathChainItem(v)?;
*slot=v; }
}
data.insert("amath_chain".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("left_right_value"){
if !v.is_null() {
v=decode_Amath_Model_LeftRightValue(v)?;
}
data.insert("left_right_value".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_LeftRight(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::LeftRight@default")?;let mut wire=Map::new();
for mut v in data.remove("amath_chain").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRight@default", "amath_chain", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_AmathChainItem(v)?;
*slot=v; }
}
wire.insert("amath_chain".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("left_right_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRightValue(v)?;
}
wire.insert("left_right_value".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_LeftRightValue_json(input:&str)->ContractResult<Amath_Model_LeftRightValue>{let data=decode_Amath_Model_LeftRightValue(parse_json(input,"Amath::Model::LeftRightValue@default")?)?; let model:Amath_Model_LeftRightValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_LeftRightValue_json(model:&Amath_Model_LeftRightValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_LeftRightValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_LeftRightValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::LeftRightValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_LeftRightValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_LeftRightValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_LeftRightValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::LeftRightValue@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::LeftRightValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_ListItem_json(input:&str)->ContractResult<Amath_Model_ListItem>{let data=decode_Amath_Model_ListItem(parse_json(input,"Amath::Model::ListItem@default")?)?; let model:Amath_Model_ListItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_ListItem_json(model:&Amath_Model_ListItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_ListItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_ListItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::ListItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ListItem@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_ListItem(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_ListItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("rparen"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("rparen".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_ListItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::ListItem@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("rparen".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_MathSymbol_json(input:&str)->ContractResult<Amath_Model_MathSymbol>{let data=decode_Amath_Model_MathSymbol(parse_json(input,"Amath::Model::MathSymbol@default")?)?; let model:Amath_Model_MathSymbol=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_MathSymbol_json(model:&Amath_Model_MathSymbol)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_MathSymbol(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::MathSymbol@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::MathSymbol@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_MathSymbol(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_MathSymbol(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::MathSymbol@default")?;let mut wire=Map::new();
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Mod_json(input:&str)->ContractResult<Amath_Model_Mod>{let data=decode_Amath_Model_Mod(parse_json(input,"Amath::Model::Mod@default")?)?; let model:Amath_Model_Mod=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Mod_json(model:&Amath_Model_Mod)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Mod(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Mod(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Mod@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Mod@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Mod(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Mod(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Amath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Amath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Mod(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Mod@default")?;let mut wire=Map::new();
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_NegativeModOperand_json(input:&str)->ContractResult<Amath_Model_NegativeModOperand>{let data=decode_Amath_Model_NegativeModOperand(parse_json(input,"Amath::Model::NegativeModOperand@default")?)?; let model:Amath_Model_NegativeModOperand=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_NegativeModOperand_json(model:&Amath_Model_NegativeModOperand)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_NegativeModOperand(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_NegativeModOperand(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::NegativeModOperand@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal_2") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "literal_2", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal_2".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::NegativeModOperand@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_NegativeModOperand(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_NegativeModOperand(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("literal_2"){
data.insert("literal_2".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_NegativeModOperand(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::NegativeModOperand@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("literal_2").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal_2".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Numerator_json(input:&str)->ContractResult<Amath_Model_Numerator>{let data=decode_Amath_Model_Numerator(parse_json(input,"Amath::Model::Numerator@default")?)?; let model:Amath_Model_Numerator=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Numerator_json(model:&Amath_Model_Numerator)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Numerator(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Numerator(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Numerator@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("norm") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "norm", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("norm".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "table_left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "table_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_row") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "table_row", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_row".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Numerator(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Numerator(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("norm"){
data.insert("norm".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table_left"){
if !v.is_null() {
v=decode_Amath_Model_TableLeft(v)?;
}
data.insert("table_left".into(),v);}
if let Some(mut v)=data.remove("table_right"){
if !v.is_null() {
v=decode_Amath_Model_TableRight(v)?;
}
data.insert("table_right".into(),v);}
if let Some(mut v)=data.remove("table_row"){
if !v.is_null() {
v=decode_Amath_Model_TableRow(v)?;
}
data.insert("table_row".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Numerator(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Numerator@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Numerator@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("norm").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("norm".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table_left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableLeft(v)?;
}
wire.insert("table_left".into(),v);}
for mut v in data.remove("table_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRight(v)?;
}
wire.insert("table_right".into(),v);}
for mut v in data.remove("table_row").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRow(v)?;
}
wire.insert("table_row".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_PmaDocument_json(input:&str)->ContractResult<Amath_Model_PmaDocument>{let data=decode_Amath_Model_PmaDocument(parse_json(input,"Amath::Model::PmaDocument@default")?)?; let model:Amath_Model_PmaDocument=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PmaDocument@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_PmaDocument_json(model:&Amath_Model_PmaDocument)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PmaDocument@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_PmaDocument(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PmaDocument@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_PmaDocument(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::PmaDocument@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("term") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PmaDocument@default", "term", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("term".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PmaDocument@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_PmaDocument(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_PmaDocument(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("term"){
if !v.is_null() {
v=decode_Amath_Model_Term(v)?;
}
data.insert("term".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_PmaDocument(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::PmaDocument@default")?;let mut wire=Map::new();
for mut v in data.remove("term").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Term(v)?;
}
wire.insert("term".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Power_json(input:&str)->ContractResult<Amath_Model_Power>{let data=decode_Amath_Model_Power(parse_json(input,"Amath::Model::Power@default")?)?; let model:Amath_Model_Power=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Power@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Power_json(model:&Amath_Model_Power)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Power@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Power(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Power@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Power(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Power@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Power@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Power@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Power(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Power(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Power(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Power@default")?;let mut wire=Map::new();
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_PowerBase_json(input:&str)->ContractResult<Amath_Model_PowerBase>{let data=decode_Amath_Model_PowerBase(parse_json(input,"Amath::Model::PowerBase@default")?)?; let model:Amath_Model_PowerBase=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_PowerBase_json(model:&Amath_Model_PowerBase)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_PowerBase(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_PowerBase(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::PowerBase@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "unary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_PowerBase(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_PowerBase(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_class"){
data.insert("unary_class".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_PowerBase(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::PowerBase@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::PowerBase@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_class".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_PowerValue_json(input:&str)->ContractResult<Amath_Model_PowerValue>{let data=decode_Amath_Model_PowerValue(parse_json(input,"Amath::Model::PowerValue@default")?)?; let model:Amath_Model_PowerValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_PowerValue_json(model:&Amath_Model_PowerValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_PowerValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_PowerValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::PowerValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_PowerValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_PowerValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_PowerValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::PowerValue@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::PowerValue@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_RgbColor_json(input:&str)->ContractResult<Amath_Model_RgbColor>{let data=decode_Amath_Model_RgbColor(parse_json(input,"Amath::Model::RgbColor@default")?)?; let model:Amath_Model_RgbColor=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_RgbColor_json(model:&Amath_Model_RgbColor)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_RgbColor(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_RgbColor(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::RgbColor@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::RgbColor@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_RgbColor(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_RgbColor(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_RgbColor(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::RgbColor@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Sequence_json(input:&str)->ContractResult<Amath_Model_Sequence>{let data=decode_Amath_Model_Sequence(parse_json(input,"Amath::Model::Sequence@default")?)?; let model:Amath_Model_Sequence=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Sequence_json(model:&Amath_Model_Sequence)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Sequence(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Sequence(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Sequence@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Sequence(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Sequence(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Sequence(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Sequence@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Sequence@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Table_json(input:&str)->ContractResult<Amath_Model_Table>{let data=decode_Amath_Model_Table(parse_json(input,"Amath::Model::Table@default")?)?; let model:Amath_Model_Table=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Table_json(model:&Amath_Model_Table)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Table(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Table(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Table@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("norm") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "norm", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("norm".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "table_left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "table_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_row") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "table_row", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_row".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Table@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Table(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Table(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("norm"){
data.insert("norm".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("table_left"){
if !v.is_null() {
v=decode_Amath_Model_TableLeft(v)?;
}
data.insert("table_left".into(),v);}
if let Some(mut v)=data.remove("table_right"){
if !v.is_null() {
v=decode_Amath_Model_TableRight(v)?;
}
data.insert("table_right".into(),v);}
if let Some(mut v)=data.remove("table_row"){
if !v.is_null() {
v=decode_Amath_Model_TableRow(v)?;
}
data.insert("table_row".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Table(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Table@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("norm").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("norm".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("table_left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableLeft(v)?;
}
wire.insert("table_left".into(),v);}
for mut v in data.remove("table_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRight(v)?;
}
wire.insert("table_right".into(),v);}
for mut v in data.remove("table_row").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRow(v)?;
}
wire.insert("table_row".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_TableLeft_json(input:&str)->ContractResult<Amath_Model_TableLeft>{let data=decode_Amath_Model_TableLeft(parse_json(input,"Amath::Model::TableLeft@default")?)?; let model:Amath_Model_TableLeft=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_TableLeft_json(model:&Amath_Model_TableLeft)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_TableLeft(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_TableLeft(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::TableLeft@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "table_left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableLeft@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_TableLeft(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_TableLeft(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("table_left"){
if !v.is_null() {
v=decode_Amath_Model_TableLeft(v)?;
}
data.insert("table_left".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_TableLeft(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::TableLeft@default")?;let mut wire=Map::new();
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("table_left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableLeft(v)?;
}
wire.insert("table_left".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_TableRight_json(input:&str)->ContractResult<Amath_Model_TableRight>{let data=decode_Amath_Model_TableRight(parse_json(input,"Amath::Model::TableRight@default")?)?; let model:Amath_Model_TableRight=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_TableRight_json(model:&Amath_Model_TableRight)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_TableRight(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_TableRight(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::TableRight@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "table_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRight@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_TableRight(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_TableRight(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("table_right"){
if !v.is_null() {
v=decode_Amath_Model_TableRight(v)?;
}
data.insert("table_right".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_TableRight(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::TableRight@default")?;let mut wire=Map::new();
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("table_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TableRight(v)?;
}
wire.insert("table_right".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_TableRow_json(input:&str)->ContractResult<Amath_Model_TableRow>{let data=decode_Amath_Model_TableRow(parse_json(input,"Amath::Model::TableRow@default")?)?; let model:Amath_Model_TableRow=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_TableRow_json(model:&Amath_Model_TableRow)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_TableRow(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_TableRow(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::TableRow@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("open_tr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "open_tr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_tr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("tds_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "tds_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("tds_list".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TableRow@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_TableRow(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_TableRow(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("open_tr"){
data.insert("open_tr".into(),v);}
if let Some(mut v)=data.remove("tds_list"){
if !v.is_null() {
v=decode_Amath_Model_TdsList(v)?;
}
data.insert("tds_list".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_TableRow(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::TableRow@default")?;let mut wire=Map::new();
for mut v in data.remove("open_tr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("open_tr".into(),v);}
for mut v in data.remove("tds_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_TdsList(v)?;
}
wire.insert("tds_list".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Td_json(input:&str)->ContractResult<Amath_Model_Td>{let data=decode_Amath_Model_Td(parse_json(input,"Amath::Model::Td@default")?)?; let model:Amath_Model_Td=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Td_json(model:&Amath_Model_Td)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Td(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Td(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Td@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Td@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Td(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Td(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Amath_Model_Expr(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Amath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Td(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Td@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Expr(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_TdsList_json(input:&str)->ContractResult<Amath_Model_TdsList>{let data=decode_Amath_Model_TdsList(parse_json(input,"Amath::Model::TdsList@default")?)?; let model:Amath_Model_TdsList=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TdsList@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_TdsList_json(model:&Amath_Model_TdsList)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TdsList@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_TdsList(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::TdsList@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_TdsList(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::TdsList@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("td") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TdsList@default", "td", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("td".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::TdsList@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_TdsList(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_TdsList(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("td"){
if !v.is_null() {
v=decode_Amath_Model_Td(v)?;
}
data.insert("td".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_TdsList(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::TdsList@default")?;let mut wire=Map::new();
for mut v in data.remove("td").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Td(v)?;
}
wire.insert("td".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Term_json(input:&str)->ContractResult<Amath_Model_Term>{let data=decode_Amath_Model_Term(parse_json(input,"Amath::Model::Term@default")?)?; let model:Amath_Model_Term=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Term_json(model:&Amath_Model_Term)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Term(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Term(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Term@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("diff") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "diff", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diff".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "unit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unit".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Term@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Term(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Term(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("diff"){
if !v.is_null() {
v=decode_Amath_Model_Diff(v)?;
}
data.insert("diff".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unit"){
if !v.is_null() {
v=decode_Amath_Model_Unit(v)?;
}
data.insert("unit".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Term(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Term@default")?;let mut wire=Map::new();
for mut v in data.remove("diff").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Diff(v)?;
}
wire.insert("diff".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unit(v)?;
}
wire.insert("unit".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Ternary_json(input:&str)->ContractResult<Amath_Model_Ternary>{let data=decode_Amath_Model_Ternary(parse_json(input,"Amath::Model::Ternary@default")?)?; let model:Amath_Model_Ternary=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Ternary_json(model:&Amath_Model_Ternary)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Ternary(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Ternary(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Ternary@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("third_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "third_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("third_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Ternary@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Ternary(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Ternary(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("third_value"){
if !v.is_null() {
v=decode_Amath_Model_ThirdValue(v)?;
}
data.insert("third_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Ternary(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Ternary@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("third_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ThirdValue(v)?;
}
wire.insert("third_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Text_json(input:&str)->ContractResult<Amath_Model_Text>{let data=decode_Amath_Model_Text(parse_json(input,"Amath::Model::Text@default")?)?; let model:Amath_Model_Text=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Text_json(model:&Amath_Model_Text)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Text(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Text(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Text@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("empty_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "empty_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty_list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Text@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Text(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Text(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("empty_list"){
data.insert("empty_list".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Text(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Text@default")?;let mut wire=Map::new();
for mut v in data.remove("empty_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty_list".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_ThirdValue_json(input:&str)->ContractResult<Amath_Model_ThirdValue>{let data=decode_Amath_Model_ThirdValue(parse_json(input,"Amath::Model::ThirdValue@default")?)?; let model:Amath_Model_ThirdValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_ThirdValue_json(model:&Amath_Model_ThirdValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_ThirdValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_ThirdValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::ThirdValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mod") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "mod", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mod".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::ThirdValue@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_ThirdValue(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_ThirdValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Amath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("color_value"){
if !v.is_null() {
v=decode_Amath_Model_ColorValue(v)?;
}
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Amath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("left"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mod"){
if !v.is_null() {
v=decode_Amath_Model_Mod(v)?;
}
data.insert("mod".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("right"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Amath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Amath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("ternary"){
if !v.is_null() {
v=decode_Amath_Model_Ternary(v)?;
}
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_ThirdValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::ThirdValue@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_ColorValue(v)?;
}
wire.insert("color_value".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("left".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mod").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Mod(v)?;
}
wire.insert("mod".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("right".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Ternary(v)?;
}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Unary_json(input:&str)->ContractResult<Amath_Model_Unary>{let data=decode_Amath_Model_Unary(parse_json(input,"Amath::Model::Unary@default")?)?; let model:Amath_Model_Unary=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Unary_json(model:&Amath_Model_Unary)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Unary(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Unary(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Unary@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "base_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "binary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bold_fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "bold_fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bold_fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma_separated") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "comma_separated", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma_separated".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "fonts_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "fonts_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "power_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slash") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "slash", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slash".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "unary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "uncertainty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unitsml") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "unitsml", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unitsml".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Unary(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Unary(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Amath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("base_value"){
if !v.is_null() {
v=decode_Amath_Model_BaseValue(v)?;
}
data.insert("base_value".into(),v);}
if let Some(mut v)=data.remove("binary_class"){
data.insert("binary_class".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("bold_fonts"){
data.insert("bold_fonts".into(),v);}
if let Some(mut v)=data.remove("comma_separated"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
data.insert("comma_separated".into(),v);}
if let Some(mut v)=data.remove("fonts_class"){
data.insert("fonts_class".into(),v);}
if let Some(mut v)=data.remove("fonts_value"){
if !v.is_null() {
v=decode_Amath_Model_FontsValue(v)?;
}
data.insert("fonts_value".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Amath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Amath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Amath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("power_value"){
if !v.is_null() {
v=decode_Amath_Model_PowerValue(v)?;
}
data.insert("power_value".into(),v);}
if let Some(mut v)=data.remove("slash"){
data.insert("slash".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Amath_Model_MathSymbol(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
if !v.is_null() {
v=decode_Amath_Model_Text(v)?;
}
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary"){
if !v.is_null() {
v=decode_Amath_Model_Unary(v)?;
}
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_class"){
data.insert("unary_class".into(),v);}
if let Some(mut v)=data.remove("uncertainty"){
if !v.is_null() {
v=decode_Amath_Model_Uncertainty(v)?;
}
data.insert("uncertainty".into(),v);}
if let Some(mut v)=data.remove("unitsml"){
if !v.is_null() {
v=decode_Amath_Model_Unitsml(v)?;
}
data.insert("unitsml".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Unary(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Unary@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("base_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_BaseValue(v)?;
}
wire.insert("base_value".into(),v);}
for mut v in data.remove("binary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_class".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("bold_fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bold_fonts".into(),v);}
for mut v in data.remove("comma_separated").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Amath::Model::Unary@default", "comma_separated", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Amath_Model_CommaSeparatedItem(v)?;
*slot=v; }
}
wire.insert("comma_separated".into(),v);}
for mut v in data.remove("fonts_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts_class".into(),v);}
for mut v in data.remove("fonts_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_FontsValue(v)?;
}
wire.insert("fonts_value".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("power_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_PowerValue(v)?;
}
wire.insert("power_value".into(),v);}
for mut v in data.remove("slash").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("slash".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_MathSymbol(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Text(v)?;
}
wire.insert("text".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unary(v)?;
}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_class".into(),v);}
for mut v in data.remove("uncertainty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Uncertainty(v)?;
}
wire.insert("uncertainty".into(),v);}
for mut v in data.remove("unitsml").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unitsml(v)?;
}
wire.insert("unitsml".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Uncertainty_json(input:&str)->ContractResult<Amath_Model_Uncertainty>{let data=decode_Amath_Model_Uncertainty(parse_json(input,"Amath::Model::Uncertainty@default")?)?; let model:Amath_Model_Uncertainty=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Uncertainty_json(model:&Amath_Model_Uncertainty)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Uncertainty(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Uncertainty(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Uncertainty@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("concise_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "concise_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("concise_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("coverage_factor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "coverage_factor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("coverage_factor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("coverage_probability") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "coverage_probability", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("coverage_probability".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("distribution") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "distribution", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("distribution".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("interval_kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "interval_kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("interval_kind".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("lower") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "lower", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("lower".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("magnitude") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "magnitude", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("magnitude".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("plus_minus") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "plus_minus", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("plus_minus".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("relative_magnitude") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "relative_magnitude", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("relative_magnitude".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("shared_exponent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "shared_exponent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("shared_exponent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("uncertainty_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "uncertainty_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("uncertainty_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "unit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unit".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("upper") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "upper", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("upper".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Uncertainty@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Uncertainty(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Uncertainty(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("concise_digits"){
data.insert("concise_digits".into(),v);}
if let Some(mut v)=data.remove("coverage_factor"){
data.insert("coverage_factor".into(),v);}
if let Some(mut v)=data.remove("coverage_probability"){
data.insert("coverage_probability".into(),v);}
if let Some(mut v)=data.remove("distribution"){
data.insert("distribution".into(),v);}
if let Some(mut v)=data.remove("interval_kind"){
data.insert("interval_kind".into(),v);}
if let Some(mut v)=data.remove("lower"){
data.insert("lower".into(),v);}
if let Some(mut v)=data.remove("magnitude"){
data.insert("magnitude".into(),v);}
if let Some(mut v)=data.remove("plus_minus"){
data.insert("plus_minus".into(),v);}
if let Some(mut v)=data.remove("relative_magnitude"){
data.insert("relative_magnitude".into(),v);}
if let Some(mut v)=data.remove("shared_exponent"){
data.insert("shared_exponent".into(),v);}
if let Some(mut v)=data.remove("uncertainty_class"){
data.insert("uncertainty_class".into(),v);}
if let Some(mut v)=data.remove("unit"){
if !v.is_null() {
v=decode_Amath_Model_Unit(v)?;
}
data.insert("unit".into(),v);}
if let Some(mut v)=data.remove("upper"){
data.insert("upper".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Uncertainty(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Uncertainty@default")?;let mut wire=Map::new();
for mut v in data.remove("concise_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("concise_digits".into(),v);}
for mut v in data.remove("coverage_factor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("coverage_factor".into(),v);}
for mut v in data.remove("coverage_probability").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("coverage_probability".into(),v);}
for mut v in data.remove("distribution").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("distribution".into(),v);}
for mut v in data.remove("interval_kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("interval_kind".into(),v);}
for mut v in data.remove("lower").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("lower".into(),v);}
for mut v in data.remove("magnitude").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("magnitude".into(),v);}
for mut v in data.remove("plus_minus").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("plus_minus".into(),v);}
for mut v in data.remove("relative_magnitude").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("relative_magnitude".into(),v);}
for mut v in data.remove("shared_exponent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("shared_exponent".into(),v);}
for mut v in data.remove("uncertainty_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("uncertainty_class".into(),v);}
for mut v in data.remove("unit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Amath_Model_Unit(v)?;
}
wire.insert("unit".into(),v);}
for mut v in data.remove("upper").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("upper".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Unit_json(input:&str)->ContractResult<Amath_Model_Unit>{let data=decode_Amath_Model_Unit(parse_json(input,"Amath::Model::Unit@default")?)?; let model:Amath_Model_Unit=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Unit_json(model:&Amath_Model_Unit)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Unit(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Unit(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Unit@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("empty_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "empty_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty_list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("quantity") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "quantity", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("quantity".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unit@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Unit(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Unit(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("empty_list"){
data.insert("empty_list".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("quantity"){
data.insert("quantity".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Unit(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Unit@default")?;let mut wire=Map::new();
for mut v in data.remove("empty_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty_list".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("quantity").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("quantity".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Amath_Model_Unitsml_json(input:&str)->ContractResult<Amath_Model_Unitsml>{let data=decode_Amath_Model_Unitsml(parse_json(input,"Amath::Model::Unitsml@default")?)?; let model:Amath_Model_Unitsml=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Amath_Model_Unitsml_json(model:&Amath_Model_Unitsml)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "", "/output",e))?;serde_json::to_string(&encode_Amath_Model_Unitsml(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Amath_Model_Unitsml(input:Value)->ContractResult<Value>{let mut wire=object(input,"Amath::Model::Unitsml@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("empty_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "empty_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty_list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Amath::Model::Unitsml@default", "", "/input", "unknown wire key"));} hydrate_Amath_Model_Unitsml(data) }
#[allow(non_snake_case)]
fn hydrate_Amath_Model_Unitsml(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("empty_list"){
data.insert("empty_list".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Amath_Model_Unitsml(input:Value)->ContractResult<Value>{let mut data=object(input,"Amath::Model::Unitsml@default")?;let mut wire=Map::new();
for mut v in data.remove("empty_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty_list".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
