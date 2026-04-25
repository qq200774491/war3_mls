use mlua::prelude::*;
use serde_json::{Map, Number, Value as Jv};

pub fn install_json_lib(lua: &Lua) -> LuaResult<()> {
    let json = lua.create_table()?;
    json.set("encode", lua.create_function(json_encode)?)?;
    json.set("decode", lua.create_function(json_decode)?)?;
    lua.globals().set("json", json)?;
    Ok(())
}

fn json_encode(_: &Lua, value: LuaValue) -> LuaResult<String> {
    let v = lua_to_json(&value);
    serde_json::to_string(&v)
        .map_err(|e| LuaError::RuntimeError(format!("json.encode: {e}")))
}

fn json_decode(lua: &Lua, value: LuaValue) -> LuaResult<LuaValue> {
    let s = match value {
        LuaValue::Nil => return Ok(LuaValue::Nil),
        LuaValue::String(ref s) => match s.to_str() {
            Ok(s) if s.is_empty() => return Ok(LuaValue::Nil),
            Ok(s) => s.to_owned(),
            Err(e) => return Err(LuaError::RuntimeError(format!("json.decode: {e}"))),
        },
        _ => return Err(LuaError::RuntimeError("json.decode expects a string".into())),
    };
    let v: Jv = serde_json::from_str(&s)
        .map_err(|e| LuaError::RuntimeError(format!("json.decode: {e}")))?;
    json_to_lua(lua, &v)
}

// cjson: empty table → object, sequential 1..n → array, otherwise → object
fn is_array_table(t: &LuaTable) -> bool {
    let mut n = 0i64;
    for pair in t.pairs::<LuaValue, LuaValue>() {
        if pair.is_ok() {
            n += 1;
        }
    }
    if n == 0 {
        return false;
    }
    for i in 1..=n {
        match t.raw_get::<LuaValue>(i) {
            Ok(LuaValue::Nil) | Err(_) => return false,
            _ => {}
        }
    }
    true
}

fn lua_to_json(v: &LuaValue) -> Jv {
    match v {
        LuaValue::Nil => Jv::Null,
        LuaValue::Boolean(b) => Jv::Bool(*b),
        LuaValue::Integer(i) => Jv::Number(Number::from(*i)),
        LuaValue::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                return Jv::Null;
            }
            if *n == n.floor() && n.abs() < 1e15 {
                return Jv::Number(Number::from(*n as i64));
            }
            Number::from_f64(*n)
                .map(Jv::Number)
                .unwrap_or(Jv::Null)
        }
        LuaValue::String(s) => Jv::String(lua_bytes_to_string(s)),
        LuaValue::Table(t) => {
            if is_array_table(t) {
                let len = t.raw_len();
                let mut arr = Vec::with_capacity(len);
                for i in 1..=len {
                    let v: LuaValue = t.raw_get(i as i64).unwrap_or(LuaValue::Nil);
                    arr.push(lua_to_json(&v));
                }
                Jv::Array(arr)
            } else {
                let mut map = Map::new();
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    if let Ok((k, v)) = pair {
                        let key = match &k {
                            LuaValue::String(s) => lua_bytes_to_string(s),
                            LuaValue::Integer(i) => i.to_string(),
                            LuaValue::Number(n) if *n == n.floor() && n.abs() < 1e15 => {
                                format!("{}", *n as i64)
                            }
                            LuaValue::Number(n) => format!("{n}"),
                            _ => continue,
                        };
                        map.insert(key, lua_to_json(&v));
                    }
                }
                Jv::Object(map)
            }
        }
        _ => Jv::Null,
    }
}

fn json_to_lua(lua: &Lua, v: &Jv) -> LuaResult<LuaValue> {
    Ok(match v {
        Jv::Null => LuaValue::Nil,
        Jv::Bool(b) => LuaValue::Boolean(*b),
        Jv::Number(n) => {
            if let Some(i) = n.as_i64() {
                LuaValue::Integer(i)
            } else {
                LuaValue::Number(n.as_f64().unwrap_or(0.0))
            }
        }
        Jv::String(s) => LuaValue::String(lua.create_string(s)?),
        Jv::Array(arr) => {
            let t = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                t.raw_set(i as i64 + 1, json_to_lua(lua, v)?)?;
            }
            LuaValue::Table(t)
        }
        Jv::Object(map) => {
            let t = lua.create_table()?;
            for (k, v) in map {
                t.raw_set(lua.create_string(k.as_str())?, json_to_lua(lua, v)?)?;
            }
            LuaValue::Table(t)
        }
    })
}

fn lua_bytes_to_string(s: &mlua::String) -> String {
    match s.to_str() {
        Ok(v) => v.to_owned(),
        Err(_) => {
            let bytes = s.as_bytes();
            let mut out = String::with_capacity(bytes.len());
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < 0x80 {
                    out.push(b as char);
                    i += 1;
                } else {
                    let seq = if b >= 0xF0 {
                        4
                    } else if b >= 0xE0 {
                        3
                    } else if b >= 0xC0 {
                        2
                    } else {
                        0
                    };
                    if seq >= 2 && i + seq <= bytes.len() {
                        if let Ok(ch) = std::str::from_utf8(&bytes[i..i + seq]) {
                            out.push_str(ch);
                            i += seq;
                            continue;
                        }
                    }
                    out.push(char::from(b));
                    i += 1;
                }
            }
            out
        }
    }
}
