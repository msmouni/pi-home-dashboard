use crate::state::{
    session::{verify_session, SESSION_TIMEOUT_SECS},
    state_logout, state_try_login, AppState,
};
use crate::web::routes::PI_HOME_DASHBOARD_TEMPLATES;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use reqwest::header::CACHE_CONTROL;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn show_login(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if verify_session(&state, jar) {
        Redirect::to("/").into_response()
    } else {
        let html = tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_TEMPLATES}/login.html"))
            .await
            .unwrap_or_else(|_| "<h1>Login page missing</h1>".into());
        let mut response = Html(html).into_response();

        response.headers_mut().insert(
            CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".parse().unwrap(),
        );

        response
    }
}

pub async fn handle_login(
    State(mut state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> (CookieJar, Redirect) {
    if let Some(session_id) = state_try_login(&mut state, &form.username, &form.password) {
        let cookie = Cookie::build(("session_id", session_id.clone()))
            .max_age(time::Duration::seconds(SESSION_TIMEOUT_SECS as i64))
            .http_only(true); // more secure: not accessible from JS

        let jar = jar.add(cookie);

        (jar, Redirect::to("/"))
    } else {
        (jar, Redirect::to("/login?error=1"))
    }
}

pub async fn logout(State(mut state): State<AppState>, jar: CookieJar) -> (CookieJar, Redirect) {
    if let Some(session_cookie) = jar.get("session_id") {
        let session_id = session_cookie.value().to_string();
        state_logout(&mut state, &session_id);
    }

    // Expire cookie immediately
    let expired = Cookie::build(("session_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0));

    let jar = jar.add(expired);
    (jar, Redirect::to("/login"))
}
