/// 调用 user-service 验证 user_id（uuid string）是否是指定 family 的成员
pub async fn is_family_member(
    user_service_url: &str,
    token: &str,
    family_id: &str,
) -> bool {
    let url = format!("{}/api/v1/families/{}/members", user_service_url, family_id);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            // 能成功拿到成员列表说明当前用户就是成员（user-service 已做权限校验）
            true
        }
        _ => false,
    }
}
