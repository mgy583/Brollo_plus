path = r'c:\Users\my583\workspace\Brollo_plus\services\user-service\src\routes\families.rs'
with open(path, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. Add required imports at the top
old_imports = 'use crate::state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Router,
};'
new_imports = 'use crate::state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Router,
};
use jsonwebtoken::{DecodingKey, Validation};'

if old_imports in c:
    c = c.replace(old_imports, new_imports)
    print('imports patched')
else:
    print('imports not found')

# 2. Add Claims struct and helper functions before sub_of
old_sub_of = 'fn sub_of(h: &HeaderMap) -> String {
    h.get("x-user-sub").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}'
new_sub_of = '''#[derive(Debug, serde::Deserialize)]
struct JwtClaims {
    sub: String,
}

fn bearer_token(h: &HeaderMap) -> Option<String> {
    let v = h.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let v = v.trim();
    if v.starts_with("Bearer ") { Some(v[7..].trim().to_string()) } else { None }
}

fn sub_of(h: &HeaderMap, secret: &str) -> String {
    let token = match bearer_token(h) {
        Some(t) => t,
        None => return String::new(),
    };
    let claims = jsonwebtoken::decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    );
    match claims {
        Ok(d) => d.claims.sub,
        Err(_) => String::new(),
    }
}'''

if old_sub_of in c:
    c = c.replace(old_sub_of, new_sub_of)
    print('sub_of patched')
else:
    print('sub_of not found, showing context:')
    idx = c.find('fn sub_of')
    print(repr(c[idx:idx+150]))

with open(path, 'w', encoding='utf-8') as f:
    f.write(c)
print('done')
