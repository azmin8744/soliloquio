use actix_web::cookie::{Cookie, SameSite, time::Duration};
use async_graphql::Context;

fn build_cookie<'a>(key: &'a str, value: &'a str, duration: Duration) -> Cookie<'a> {
    Cookie::build(key, value)
        .http_only(true)
        .secure(std::env::var("SECURE_COOKIES").as_deref() == Ok("true"))
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(duration)
        .finish()
}

pub fn set_auth_cookies(ctx: &Context<'_>, access_token: &str, refresh_token: &str) {
    ctx.insert_http_header("Set-Cookie", build_cookie("access_token", access_token, Duration::hours(1)).to_string());
    ctx.append_http_header("Set-Cookie", build_cookie("refresh_token", refresh_token, Duration::days(7)).to_string());
}

pub fn set_access_cookie(ctx: &Context<'_>, access_token: &str) {
    ctx.insert_http_header("Set-Cookie", build_cookie("access_token", access_token, Duration::hours(1)).to_string());
}

pub fn clear_auth_cookies(ctx: &Context<'_>) {
    ctx.insert_http_header("Set-Cookie", build_cookie("access_token", "", Duration::seconds(0)).to_string());
    ctx.append_http_header("Set-Cookie", build_cookie("refresh_token", "", Duration::seconds(0)).to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{EmptyMutation, EmptySubscription, Object, Request, Schema};

    struct TestQuery;

    #[Object]
    impl TestQuery {
        async fn set_auth(&self, ctx: &Context<'_>) -> bool {
            set_auth_cookies(ctx, "acc_val", "ref_val");
            true
        }
        async fn set_access(&self, ctx: &Context<'_>) -> bool {
            set_access_cookie(ctx, "acc_val");
            true
        }
        async fn clear(&self, ctx: &Context<'_>) -> bool {
            clear_auth_cookies(ctx);
            true
        }
    }

    fn make_schema() -> Schema<TestQuery, EmptyMutation, EmptySubscription> {
        Schema::build(TestQuery, EmptyMutation, EmptySubscription).finish()
    }

    fn get_set_cookies(res: &async_graphql::Response) -> Vec<String> {
        res.http_headers
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    fn find_cookie(cookies: &[String], name: &str) -> String {
        cookies
            .iter()
            .find(|c| c.starts_with(&format!("{}=", name)))
            .cloned()
            .unwrap_or_else(|| panic!("cookie '{name}' not found"))
    }

    #[tokio::test]
    async fn test_set_auth_cookies_sets_two_cookies() {
        let res = make_schema().execute(Request::new("{ setAuth }")).await;
        assert_eq!(get_set_cookies(&res).len(), 2);
    }

    #[tokio::test]
    async fn test_set_auth_cookies_access_key_and_value() {
        let res = make_schema().execute(Request::new("{ setAuth }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.starts_with("access_token=acc_val"));
    }

    #[tokio::test]
    async fn test_set_auth_cookies_access_duration() {
        let res = make_schema().execute(Request::new("{ setAuth }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.contains("Max-Age=3600")); // 1 hour
    }

    #[tokio::test]
    async fn test_set_auth_cookies_refresh_key_and_value() {
        let res = make_schema().execute(Request::new("{ setAuth }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "refresh_token");
        assert!(cookie.starts_with("refresh_token=ref_val"));
    }

    #[tokio::test]
    async fn test_set_auth_cookies_refresh_duration() {
        let res = make_schema().execute(Request::new("{ setAuth }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "refresh_token");
        assert!(cookie.contains("Max-Age=604800")); // 7 days
    }

    #[tokio::test]
    async fn test_set_access_cookie_sets_one_cookie() {
        let res = make_schema().execute(Request::new("{ setAccess }")).await;
        assert_eq!(get_set_cookies(&res).len(), 1);
    }

    #[tokio::test]
    async fn test_set_access_cookie_key_and_value() {
        let res = make_schema().execute(Request::new("{ setAccess }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.starts_with("access_token=acc_val"));
    }

    #[tokio::test]
    async fn test_set_access_cookie_duration() {
        let res = make_schema().execute(Request::new("{ setAccess }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.contains("Max-Age=3600")); // 1 hour
    }

    #[tokio::test]
    async fn test_clear_auth_cookies_sets_two_cookies() {
        let res = make_schema().execute(Request::new("{ clear }")).await;
        assert_eq!(get_set_cookies(&res).len(), 2);
    }

    #[tokio::test]
    async fn test_clear_auth_cookies_access_key_and_empty_value() {
        let res = make_schema().execute(Request::new("{ clear }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.starts_with("access_token=;"));
    }

    #[tokio::test]
    async fn test_clear_auth_cookies_access_duration_zero() {
        let res = make_schema().execute(Request::new("{ clear }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "access_token");
        assert!(cookie.contains("Max-Age=0"));
    }

    #[tokio::test]
    async fn test_clear_auth_cookies_refresh_key_and_empty_value() {
        let res = make_schema().execute(Request::new("{ clear }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "refresh_token");
        assert!(cookie.starts_with("refresh_token=;"));
    }

    #[tokio::test]
    async fn test_clear_auth_cookies_refresh_duration_zero() {
        let res = make_schema().execute(Request::new("{ clear }")).await;
        let cookie = find_cookie(&get_set_cookies(&res), "refresh_token");
        assert!(cookie.contains("Max-Age=0"));
    }
}
