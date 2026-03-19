use crate::state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Router,
};
use common::{err, ok, request_id_from_headers};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/families", post(create_family))
        .route("/families/mine", get(my_family))
        .route("/families/join", post(join_family))
        .route("/families/:id", get(get_family).patch(update_family).delete(dissolve_family))
        .route("/families/:id/members", get(list_members))
        .route("/families/:id/members/:uid", patch(update_member).delete(remove_member))
        .route("/families/:id/invite", post(refresh_invite_code))
        .route("/families/:id/leave", post(leave_family))
}

#[derive(Deserialize)]
struct CreateFamilyReq {
    name: String,
    default_currency: Option<String>,
    description: Option<String>,
    nickname: Option<String>,
}
#[derive(Deserialize)]
struct UpdateFamilyReq {
    name: Option<String>,
    default_currency: Option<String>,
    description: Option<String>,
    avatar: Option<String>,
}
#[derive(Deserialize)]
struct JoinFamilyReq { invite_code: String, nickname: Option<String> }
#[derive(Deserialize)]
struct UpdateMemberReq { role: Option<String>, nickname: Option<String> }

#[derive(sqlx::FromRow)]
struct FamilyRow {
    id: i32, uuid: Uuid, name: String, invite_code: String, owner_id: i32,
    default_currency: String, description: Option<String>, avatar: Option<String>,
    created_at: time::PrimitiveDateTime,
    #[allow(dead_code)] updated_at: time::PrimitiveDateTime,
}
#[derive(sqlx::FromRow)]
struct MemberRow {
    id: i32, family_id: i32, user_id: i32, role: String, nickname: Option<String>,
    joined_at: time::PrimitiveDateTime, username: String, email: String,
    full_name: Option<String>, user_uuid: Uuid,
}
#[derive(Serialize)]
struct FamilyDto {
    id: i32, uuid: Uuid, name: String, invite_code: String, owner_id: i32,
    default_currency: String, description: Option<String>, avatar: Option<String>,
    member_count: i64, created_at: String,
}
#[derive(Serialize)]
struct MemberDto {
    id: i32, family_id: i32, user_id: i32, user_uuid: Uuid,
    username: String, email: String, full_name: Option<String>,
    role: String, nickname: Option<String>, joined_at: String,
}

fn to_fdto(f: FamilyRow, c: i64) -> FamilyDto {
    FamilyDto { id:f.id, uuid:f.uuid, name:f.name, invite_code:f.invite_code,
        owner_id:f.owner_id, default_currency:f.default_currency,
        description:f.description, avatar:f.avatar, member_count:c,
        created_at:f.created_at.assume_utc().to_string() }
}
fn to_mdto(m: MemberRow) -> MemberDto {
    MemberDto { id:m.id, family_id:m.family_id, user_id:m.user_id, user_uuid:m.user_uuid,
        username:m.username, email:m.email, full_name:m.full_name, role:m.role,
        nickname:m.nickname, joined_at:m.joined_at.assume_utc().to_string() }
}
fn gen_code() -> String {
    use rand::Rng; let mut r = rand::rng();
    (0..6).map(|_| { let i = r.random_range(0..36usize);
        if i < 10 { (b'0'+i as u8) as char } else { (b'A'+(i-10) as u8) as char }
    }).collect()
}
async fn uid_from_sub(db: &sqlx::PgPool, sub: &str) -> Option<i32> {
    let uuid = Uuid::parse_str(sub).ok()?;
    sqlx::query_scalar::<_,i32>("SELECT id FROM users WHERE uuid=$1")
        .bind(uuid).fetch_optional(db).await.ok().flatten()
}
async fn get_role(db: &sqlx::PgPool, fid: i32, uid: i32) -> Option<String> {
    sqlx::query_scalar::<_,String>(
        "SELECT role FROM family_members WHERE family_id=$1 AND user_id=$2"
    ).bind(fid).bind(uid).fetch_optional(db).await.ok().flatten()
}
async fn get_cnt(db: &sqlx::PgPool, fid: i32) -> i64 {
    sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM family_members WHERE family_id=$1")
        .bind(fid).fetch_one(db).await.unwrap_or(0)
}
fn sub_of(h: &HeaderMap) -> String {
    h.get("x-user-sub").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}
type Res = Result<axum::Json<serde_json::Value>,(StatusCode,axum::Json<common::ApiError>)>;
type ResC = Result<(StatusCode,axum::Json<serde_json::Value>),(StatusCode,axum::Json<common::ApiError>)>;

async fn create_family(State(s): State<AppState>, h: HeaderMap, Json(req): Json<CreateFamilyReq>) -> ResC {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    if req.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "INVALID_INPUT", "家庭名称不能为空", None, rid));
    }
    let ex: Option<i32> = sqlx::query_scalar(
        "SELECT family_id FROM family_members WHERE user_id=$1 LIMIT 1"
    ).bind(uid).fetch_optional(&s.db).await.unwrap_or(None);
    if ex.is_some() {
        return Err(err(StatusCode::CONFLICT, "ALREADY_IN_FAMILY", "你已在某个家庭中", None, rid));
    }
    let code = loop {
        let c = gen_code();
        let d: Option<i32> = sqlx::query_scalar("SELECT id FROM families WHERE invite_code=$1")
            .bind(&c).fetch_optional(&s.db).await.unwrap_or(None);
        if d.is_none() { break c; }
    };
    let cur = req.default_currency.unwrap_or_else(|| "CNY".to_string());
    let fam: FamilyRow = sqlx::query_as(
        "INSERT INTO families (name,invite_code,owner_id,default_currency,description) VALUES ($1,$2,$3,$4,$5) RETURNING *"
    ).bind(&req.name).bind(&code).bind(uid).bind(&cur).bind(&req.description)
    .fetch_one(&s.db).await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "创建失败", None, rid.clone()))?;
    sqlx::query("INSERT INTO family_members (family_id,user_id,role,nickname) VALUES ($1,$2,'owner',$3)")
        .bind(fam.id).bind(uid).bind(&req.nickname).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "初始化成员失败", None, rid.clone()))?;
    Ok((StatusCode::CREATED, axum::Json(serde_json::to_value(
        ok(serde_json::json!({"family": to_fdto(fam, 1)}), "创建成功", rid)
    ).unwrap())))
}

async fn my_family(State(s): State<AppState>, h: HeaderMap) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let row = sqlx::query_as::<_, FamilyRow>(
        "SELECT f.* FROM families f JOIN family_members fm ON fm.family_id=f.id WHERE fm.user_id=$1 LIMIT 1"
    ).bind(uid).fetch_optional(&s.db).await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, rid.clone()))?;
    let Some(fam) = row else {
        return Ok(axum::Json(serde_json::to_value(
            ok(serde_json::json!({"family": null}), "暂无家庭", rid)
        ).unwrap()));
    };
    let c = get_cnt(&s.db, fam.id).await;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({"family": to_fdto(fam, c)}), "ok", rid)).unwrap()))
}

async fn get_family(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    if get_role(&s.db, id, uid).await.is_none() {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权访问", None, rid));
    }
    let fam: FamilyRow = sqlx::query_as("SELECT * FROM families WHERE id=$1")
        .bind(id).fetch_one(&s.db).await
        .map_err(|_| err(StatusCode::NOT_FOUND, "NOT_FOUND", "家庭不存在", None, rid.clone()))?;
    let c = get_cnt(&s.db, id).await;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({"family": to_fdto(fam, c)}), "ok", rid)).unwrap()))
}

async fn update_family(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>, Json(req): Json<UpdateFamilyReq>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, id, uid).await
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作", None, rid.clone()))?;
    if !matches!(role.as_str(), "owner" | "admin") {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "需要管理员权限", None, rid));
    }
    sqlx::query(
        "UPDATE families SET name=COALESCE($1,name), default_currency=COALESCE($2,default_currency), description=COALESCE($3,description), avatar=COALESCE($4,avatar), updated_at=NOW() WHERE id=$5"
    ).bind(&req.name).bind(&req.default_currency).bind(&req.description).bind(&req.avatar).bind(id)
    .execute(&s.db).await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "更新失败", None, rid.clone()))?;
    let fam: FamilyRow = sqlx::query_as("SELECT * FROM families WHERE id=$1").bind(id).fetch_one(&s.db).await
        .map_err(|_| err(StatusCode::NOT_FOUND, "NOT_FOUND", "家庭不存在", None, rid.clone()))?;
    let c = get_cnt(&s.db, id).await;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({"family": to_fdto(fam, c)}), "更新成功", rid)).unwrap()))
}

async fn join_family(State(s): State<AppState>, h: HeaderMap, Json(req): Json<JoinFamilyReq>) -> ResC {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let ex: Option<i32> = sqlx::query_scalar(
        "SELECT family_id FROM family_members WHERE user_id=$1 LIMIT 1"
    ).bind(uid).fetch_optional(&s.db).await.unwrap_or(None);
    if ex.is_some() {
        return Err(err(StatusCode::CONFLICT, "ALREADY_IN_FAMILY", "你已在某个家庭中", None, rid));
    }
    let fam: Option<FamilyRow> = sqlx::query_as("SELECT * FROM families WHERE invite_code=$1")
        .bind(req.invite_code.to_uppercase()).fetch_optional(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, rid.clone()))?;
    let Some(fam) = fam else {
        return Err(err(StatusCode::NOT_FOUND, "INVALID_CODE", "邀请码无效", None, rid));
    };
    sqlx::query("INSERT INTO family_members (family_id,user_id,role,nickname) VALUES ($1,$2,'member',$3)")
        .bind(fam.id).bind(uid).bind(&req.nickname).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "加入失败", None, rid.clone()))?;
    let c = get_cnt(&s.db, fam.id).await;
    Ok((StatusCode::CREATED, axum::Json(serde_json::to_value(
        ok(serde_json::json!({"family": to_fdto(fam, c)}), "加入成功", rid)
    ).unwrap())))
}

async fn list_members(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    if get_role(&s.db, id, uid).await.is_none() {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权访问", None, rid));
    }
    let rows: Vec<MemberRow> = sqlx::query_as(
        "SELECT fm.id,fm.family_id,fm.user_id,fm.role,fm.nickname,fm.joined_at,u.username,u.email,u.full_name,u.uuid as user_uuid FROM family_members fm JOIN users u ON u.id=fm.user_id WHERE fm.family_id=$1 ORDER BY fm.joined_at"
    ).bind(id).fetch_all(&s.db).await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "数据库错误", None, rid.clone()))?;
    let members: Vec<MemberDto> = rows.into_iter().map(to_mdto).collect();
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({"members": members}), "ok", rid)).unwrap()))
}

async fn update_member(State(s): State<AppState>, h: HeaderMap, Path((fid,tuid)): Path<(i32,i32)>, Json(req): Json<UpdateMemberReq>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, fid, uid).await
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作", None, rid.clone()))?;
    if req.role.is_some() && role != "owner" {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "只有家长可修改角色", None, rid));
    }
    sqlx::query(
        "UPDATE family_members SET role=COALESCE($1,role), nickname=COALESCE($2,nickname) WHERE family_id=$3 AND user_id=$4"
    ).bind(&req.role).bind(&req.nickname).bind(fid).bind(tuid).execute(&s.db).await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "更新失败", None, rid.clone()))?;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({}), "更新成功", rid)).unwrap()))
}

async fn remove_member(State(s): State<AppState>, h: HeaderMap, Path((fid,tuid)): Path<(i32,i32)>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, fid, uid).await
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作", None, rid.clone()))?;
    if !matches!(role.as_str(), "owner" | "admin") {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "需要管理员权限", None, rid));
    }
    if get_role(&s.db, fid, tuid).await.as_deref() == Some("owner") {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "不能移除家长", None, rid));
    }
    sqlx::query("DELETE FROM family_members WHERE family_id=$1 AND user_id=$2")
        .bind(fid).bind(tuid).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "移除失败", None, rid.clone()))?;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({}), "移除成功", rid)).unwrap()))
}

async fn refresh_invite_code(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, id, uid).await
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作", None, rid.clone()))?;
    if !matches!(role.as_str(), "owner" | "admin") {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "需要管理员权限", None, rid));
    }
    let code = loop {
        let c = gen_code();
        let d: Option<i32> = sqlx::query_scalar("SELECT id FROM families WHERE invite_code=$1")
            .bind(&c).fetch_optional(&s.db).await.unwrap_or(None);
        if d.is_none() { break c; }
    };
    sqlx::query("UPDATE families SET invite_code=$1, updated_at=NOW() WHERE id=$2")
        .bind(&code).bind(id).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "刷新失败", None, rid.clone()))?;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({"invite_code": code}), "邀请码已刷新", rid)).unwrap()))
}

async fn leave_family(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, id, uid).await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "NOT_FOUND", "你不在此家庭", None, rid.clone()))?;
    if role == "owner" {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "家长不能退出，请先转让或解散家庭", None, rid));
    }
    sqlx::query("DELETE FROM family_members WHERE family_id=$1 AND user_id=$2")
        .bind(id).bind(uid).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "退出失败", None, rid.clone()))?;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({}), "已退出家庭", rid)).unwrap()))
}

async fn dissolve_family(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i32>) -> Res {
    let rid = request_id_from_headers(&h);
    let Some(uid) = uid_from_sub(&s.db, &sub_of(&h)).await else {
        return Err(err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未认证", None, rid));
    };
    let role = get_role(&s.db, id, uid).await
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作", None, rid.clone()))?;
    if role != "owner" {
        return Err(err(StatusCode::FORBIDDEN, "FORBIDDEN", "只有家长可以解散家庭", None, rid));
    }
    sqlx::query("DELETE FROM families WHERE id=$1")
        .bind(id).execute(&s.db).await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "解散失败", None, rid.clone()))?;
    Ok(axum::Json(serde_json::to_value(ok(serde_json::json!({}), "家庭已解散", rid)).unwrap()))
}
