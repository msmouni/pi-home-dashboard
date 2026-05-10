use crate::state::AppState;
use axum_extra::extract::CookieJar;
use std::time::SystemTime;

pub const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Clone, Debug)]
pub(super) struct UserSession {
    #[allow(dead_code)]
    pub(super) username: String,
    pub(super) session_start: SystemTime,
}

pub(crate) fn verify_session(state: &AppState, jar: CookieJar) -> bool {
    if let Some(session_id) = jar.get("session_id") {
        let sessions = state.sessions.lock().unwrap();
        if let Some(user_session) = sessions.get(session_id.value()) {
            if user_session.session_start.elapsed().unwrap().as_secs() > SESSION_TIMEOUT_SECS {
                state.sessions.lock().unwrap().remove(session_id.value());
                return false;
            } else {
                return true;
            }
        }
    }

    false
}
