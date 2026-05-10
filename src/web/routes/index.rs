use crate::state::session::verify_session;
use crate::web::{routes::PI_HOME_DASHBOARD_RES, AppState};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use reqwest::header::CACHE_CONTROL;

pub async fn index(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if verify_session(&state, jar) {
        let html = tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/index.html"))
            .await
            .unwrap();
        Html(html).into_response()
    } else {
        let mut response = Html(
            tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/not_logged_in.html"))
                .await
                .unwrap(),
        )
        .into_response();

        response.headers_mut().insert(
            CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".parse().unwrap(),
        );

        response
    }
}
