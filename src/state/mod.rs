pub(crate) mod session;

use session::UserSession;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;

#[derive(Clone, Default, Debug)]
pub(crate) struct AppState {
    sessions: Arc<Mutex<HashMap<String, UserSession>>>, // session_id → UserSession
}

pub fn state_try_login(
    state: &mut AppState,
    username: &String,
    password: &String,
) -> Option<String> {
    // TODO: replace with DB check
    if username == "admin" && password == "raspberry" {
        let session_id = Uuid::new_v4().to_string();

        state.sessions.lock().unwrap().insert(
            session_id.clone(),
            UserSession {
                username: username.clone(),
                session_start: SystemTime::now(),
            },
        );

        Some(session_id)
    } else {
        None
    }
}

pub fn state_logout(state: &mut AppState, session_id: &String) {
    state.sessions.lock().unwrap().remove(session_id);
}
