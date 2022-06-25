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
        app.get_interval_duration().unwrap().as_secs()
    );

    app.pause();
    thread::sleep(time::Duration::from_secs(1));
    app.start();
    thread::sleep(time::Duration::from_secs(4));

    println!(
        "Focused for {} seconds!",
        app.get_interval_duration().unwrap().as_secs()
    );
}

struct App {
    intervals_by_type: HashMap<IntervalType, Vec<Interval>>,
    paused: bool,
    current_interval_type: IntervalType,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.paused = false;
        let intervals = self
            .intervals_by_type
            .get_mut(&self.current_interval_type)
            .unwrap();
        intervals.push(Interval::new());
    }

    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(interval) = self.get_current_interval_mut() {
            interval.end = Some(SystemTime::now());
        }
    }

    pub fn change_interval_type(&mut self, interval_type: IntervalType) {
        if let Some(interval) = self.get_current_interval_mut() {
            interval.end = Some(SystemTime::now());
        }
        self.current_interval_type = interval_type;
        self.intervals_by_type
            .get_mut(&self.current_interval_type)
            .unwrap()
            .push(Interval::new());
    }

    pub fn get_interval_duration(&self) -> Option<Duration> {
        if let Some(interval) = self.get_current_interval() {
            let duration = SystemTime::now().duration_since(interval.start).unwrap();
            Some(duration)
        } else {
            None
        }
    }

    fn get_current_interval_mut(&mut self) -> Option<&mut Interval> {
        self.intervals_by_type
            .get_mut(&self.current_interval_type)
            .unwrap()
            .first_mut()
    }

    fn get_current_interval(&self) -> Option<&Interval> {
        self.intervals_by_type
            .get(&self.current_interval_type)
            .unwrap()
            .last()
    }

    pub fn get_total_duration(&self, interval_type: IntervalType) -> Option<Duration> {
        if let Some(intervals) = self.intervals_by_type.get(&interval_type) {
            let total_duration = intervals.iter().fold(Duration::ZERO, |total, interval| {
                let interval_end = interval.end.unwrap_or(SystemTime::now());
                let interval_duration = interval_end.duration_since(interval.start).unwrap();
                total + interval_duration
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
            intervals_by_type: HashMap::new(),
            paused: true,
            current_interval_type: IntervalType::Focus,
        };
        app.intervals_by_type.insert(IntervalType::Focus, vec![]);
        app.intervals_by_type.insert(IntervalType::Rest, vec![]);
        app
    }
}

struct Interval {
    start: SystemTime,
    end: Option<SystemTime>,
}

impl Interval {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Interval {
    fn default() -> Self {
        Interval {
            start: SystemTime::now(),
            end: None,
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
enum IntervalType {
    Focus,
    Rest,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod pause {
        use super::*;

        #[test]
        fn resets_current_duration() {
            let mut app = App::new();
            app.start();
            thread::sleep(Duration::from_millis(20));
            app.pause();
            app.start();
            thread::sleep(Duration::from_millis(10));
            let duration = app.get_interval_duration().unwrap();
            assert!(
                duration.le(&Duration::from_millis(20)) && duration.gt(&Duration::from_millis(0))
            );
        }

        #[test]
        fn stops_total_duration_from_increasing() {
            let mut app = App::new();
            app.start();
            thread::sleep(Duration::from_millis(20));
            app.pause();
            let duration_before_pause: Duration =
                app.get_total_duration(IntervalType::Focus).unwrap();
            thread::sleep(Duration::from_millis(20));
            let duration_after_pause: Duration =
                app.get_total_duration(IntervalType::Focus).unwrap();
            assert_eq!(duration_before_pause, duration_after_pause);
        }

        #[test]
        fn stops_total_duration_from_increasing_only_during_pause() {
            let mut app = App::new();
            app.start();
            thread::sleep(Duration::from_millis(20));
            app.pause();
            let duration_after_session_1: Duration =
                app.get_total_duration(IntervalType::Focus).unwrap();
            thread::sleep(Duration::from_millis(20));
            dbg!(duration_after_session_1);
            app.start();
            thread::sleep(Duration::from_millis(20));
            app.pause();
            let duration_after_session_2: Duration =
                app.get_total_duration(IntervalType::Focus).unwrap();
            assert!(duration_after_session_2.gt(&duration_after_session_1));
            dbg!(duration_after_session_2);
        }
    }

    #[test]
    fn changing_interval_type_resets_current_duration() {
        todo!()
    }

    #[test]
    fn get_total_duration_totals_interval_durations() {
        todo!()
    }
}
