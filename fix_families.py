path = r'c:\Users\my583\workspace\Brollo_plus\services\user-service\src\routes\families.rs'
with open(path, 'r', encoding='utf-8') as f:
    c = f.read()
old = 'fn sub_of(h: &HeaderMap) -> String {\n    h.get("x-user-sub").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()\n}'
new = ('fn sub_of(h: &HeaderMap, secret: &str) -> String {\n'
    '    let token = match h.get(axum::http::header::AUTHORIZATION)\n'
    '        .and_then(|v| v.to_str().ok())\n'
    '        .and_then(|v| if v.starts_with("Bearer ") { Some(v[7..].trim().to_string()) } else { None })\n'
    '    {\n'
    '        Some(t) => t,\n'
    '        None => return String::new(),\n'
    '    };\n'
    '    #[derive(serde::Deserialize)]\n'
    '    struct C { sub: String }\n'
    '    jsonwebtoken::decode::<C>(&token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())\n'
    '        .map(|d| d.claims.sub)\n'
    '        .unwrap_or_default()\n'
    '}')
if old in c:
    c = c.replace(old, new)
    print('sub_of replaced')
else:
    idx = c.find('fn sub_of')
    print('not found:', repr(c[idx:idx+100]))
c = c.replace('sub_of(&h)', 'sub_of(&h, &s.jwt_secret)')
print('calls updated')
with open(path, 'w', encoding='utf-8') as f:
    f.write(c)
print('done')
