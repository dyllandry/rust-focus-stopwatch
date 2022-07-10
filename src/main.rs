use std::{
    collections::HashMap,
    thread,
    time::{self, Duration, SystemTime},
};

fn main() {
    let mut app = App::new();
    app.start();

    thread::sleep(time::Duration::from_secs(1));
    println!(
        "Focused for {} seconds!",
        app.get_current_session_duration().unwrap().as_secs()
    );

    app.pause();
    thread::sleep(time::Duration::from_secs(1));
    app.start();
    thread::sleep(time::Duration::from_secs(4));

    println!(
        "Focused for {} seconds!",
        app.get_current_session_duration().unwrap().as_secs()
    );
}

struct App {
    sessions_by_type: HashMap<SessionType, Vec<Session>>,
    paused: bool,
    current_session_type: SessionType,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.paused = false;
        let sessions = self
            .sessions_by_type
            .get_mut(&self.current_session_type)
            .unwrap();
        sessions.push(Session::new());
    }

    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(session) = self.get_current_session_mut() {
            session.end = Some(SystemTime::now());
        }
    }

    pub fn change_session_type(&mut self, session_type: SessionType) {
        if let Some(session) = self.get_current_session_mut() {
            session.end = Some(SystemTime::now());
        }
        self.current_session_type = session_type;
        self.sessions_by_type
            .get_mut(&self.current_session_type)
            .unwrap()
            .push(Session::new());
    }

    pub fn get_current_session_duration(&self) -> Option<Duration> {
        if let Some(session) = self.get_current_session() {
            let session_end = session.end.unwrap_or(SystemTime::now());
            let duration = session_end.duration_since(session.start).unwrap();
            Some(duration)
        } else {
            None
        }
    }

    fn get_current_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions_by_type
            .get_mut(&self.current_session_type)
            .unwrap()
            .first_mut()
    }

    fn get_current_session(&self) -> Option<&Session> {
        self.sessions_by_type
            .get(&self.current_session_type)
            .unwrap()
            .last()
    }

    pub fn get_session_type_total_duration(&self, session_type: SessionType) -> Option<Duration> {
        if let Some(sessions) = self.sessions_by_type.get(&session_type) {
            let total_duration = sessions.iter().fold(Duration::ZERO, |total, session| {
                let session_end = session.end.unwrap_or(SystemTime::now());
                let session_duration = session_end.duration_since(session.start).unwrap();
                total + session_duration
            });
            Some(total_duration)
        } else {
            None
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            sessions_by_type: HashMap::new(),
            paused: true,
            current_session_type: SessionType::Focus,
        };
        app.sessions_by_type.insert(SessionType::Focus, vec![]);
        app.sessions_by_type.insert(SessionType::Rest, vec![]);
        app
    }
}

struct Session {
    start: SystemTime,
    end: Option<SystemTime>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Session {
    fn default() -> Self {
        Session {
            start: SystemTime::now(),
            end: None,
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
enum SessionType {
    Focus,
    Rest,
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;

    #[test]
    fn stopwatch_duration_increases_over_time() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(20));
        let duration = app.get_current_session_duration().unwrap();
        assert!(duration.gt(&Duration::from_millis(0)));
    }

    #[test]
    fn pause_stops_duration_from_increasing() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(20));
        app.pause();
        let duration_at_pause: Duration = app.get_current_session_duration().unwrap();
        thread::sleep(Duration::from_millis(100));
        let duration_after_waiting: Duration = app.get_current_session_duration().unwrap();
        assert_eq!(duration_at_pause, duration_after_waiting);
    }

    #[test]
    fn pause_creates_new_session() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(50));
        app.pause();
        app.start();
        thread::sleep(Duration::from_millis(25));
        let duration = app.get_current_session_duration().unwrap();
        assert!(duration.le(&Duration::from_millis(50)) && duration.gt(&Duration::from_millis(10)));
    }

    #[test]
    fn total_duration_equals_all_sessions() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(20));
        let duration_1 = app.get_current_session_duration().unwrap();
        app.pause();
        app.start();
        thread::sleep(Duration::from_millis(20));
        let duration_2 = app.get_current_session_duration().unwrap();
        let total_duration = app
            .get_session_type_total_duration(SessionType::Focus)
            .unwrap();
        assert!(total_duration.gt(&duration_1.add(duration_2)));
    }

    #[test]
    fn changing_session_type_creates_new_session() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(20));
        let focus_duration = app.get_current_session_duration().unwrap();
        app.change_session_type(SessionType::Rest);
        thread::sleep(Duration::from_millis(20));
        let rest_duration = app.get_current_session_duration().unwrap();
        assert!(focus_duration.le(&Duration::from_millis(40)));
        assert!(rest_duration.le(&Duration::from_millis(40)));
    }
}
