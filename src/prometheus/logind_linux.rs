use dbus::blocking::{Connection, Proxy};
use dbus::arg::RefArg;
use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::time::Duration;

const LOGIND_SUBSYSTEM: &str = "logind";
const DBUS_OBJECT: &str = "org.freedesktop.login1";
const DBUS_PATH: &str = "/org/freedesktop/login1";

lazy_static! {
    static ref ATTR_REMOTE_VALUES: Vec<&'static str> = vec!["true", "false"];
    static ref ATTR_TYPE_VALUES: Vec<&'static str> = vec!["other", "unspecified", "tty", "x11", "wayland", "mir", "web"];
    static ref ATTR_CLASS_VALUES: Vec<&'static str> = vec!["other", "user", "greeter", "lock-screen", "background"];
    static ref SESSIONS_DESC: Desc = Desc::new(
        "node_logind_sessions",
        "Number of sessions registered in logind.",
        vec!["seat", "remote", "type", "class"],
        HashMap::new(),
    );
}

struct LogindCollector {
    logger: Logger,
}

impl LogindCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(LogindCollector { logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn std::error::Error>> {
        let c = new_dbus()?;
        collect_metrics(ch, &c)
    }
}

struct LogindDbus {
    conn: Connection,
    proxy: Proxy<'static, Connection>,
}

impl LogindDbus {
    fn list_seats(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let result: Vec<Vec<Box<dyn RefArg>>> = self.proxy.method_call(DBUS_OBJECT, "ListSeats", ())?;
        let seats: Vec<String> = result.iter().map(|seat| seat[0].as_str().unwrap().to_string()).collect();
        Ok(seats)
    }

    fn list_sessions(&self) -> Result<Vec<LogindSessionEntry>, Box<dyn std::error::Error>> {
        let result: Vec<Vec<Box<dyn RefArg>>> = self.proxy.method_call(DBUS_OBJECT, "ListSessions", ())?;
        let sessions: Vec<LogindSessionEntry> = result.iter().map(|session| {
            LogindSessionEntry {
                session_id: session[0].as_str().unwrap().to_string(),
                user_id: session[1].as_u64().unwrap() as u32,
                user_name: session[2].as_str().unwrap().to_string(),
                seat_id: session[3].as_str().unwrap().to_string(),
                session_object_path: session[4].as_str().unwrap().to_string(),
            }
        }).collect();
        Ok(sessions)
    }

    fn get_session(&self, session: &LogindSessionEntry) -> Option<LogindSession> {
        let proxy = self.conn.with_proxy(DBUS_OBJECT, &session.session_object_path, Duration::from_millis(5000));
        let remote: bool = proxy.get("Remote").ok()?;
        let session_type: String = proxy.get("Type").ok()?;
        let class: String = proxy.get("Class").ok()?;
        Some(LogindSession {
            seat: session.seat_id.clone(),
            remote: remote.to_string(),
            session_type: known_string_or_other(&session_type, &ATTR_TYPE_VALUES),
            class: known_string_or_other(&class, &ATTR_CLASS_VALUES),
        })
    }
}

struct LogindSession {
    seat: String,
    remote: String,
    session_type: String,
    class: String,
}

struct LogindSessionEntry {
    session_id: String,
    user_id: u32,
    user_name: String,
    seat_id: String,
    session_object_path: String,
}

fn new_dbus() -> Result<LogindDbus, Box<dyn std::error::Error>> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(DBUS_OBJECT, DBUS_PATH, Duration::from_millis(5000));
    Ok(LogindDbus { conn, proxy })
}

fn collect_metrics(ch: &mut dyn FnMut(Box<dyn Metric>), c: &LogindDbus) -> Result<(), Box<dyn std::error::Error>> {
    let seats = c.list_seats()?;
    let session_list = c.list_sessions()?;
    let mut sessions = HashMap::new();

    for s in session_list {
        if let Some(session) = c.get_session(&s) {
            *sessions.entry(session).or_insert(0.0) += 1.0;
        }
    }

    for &remote in &*ATTR_REMOTE_VALUES {
        for &session_type in &*ATTR_TYPE_VALUES {
            for &class in &*ATTR_CLASS_VALUES {
                for seat in &seats {
                    let count = *sessions.get(&LogindSession {
                        seat: seat.clone(),
                        remote: remote.to_string(),
                        session_type: session_type.to_string(),
                        class: class.to_string(),
                    }).unwrap_or(&0.0);
                    ch(Box::new(prometheus::Gauge::new(SESSIONS_DESC.clone(), count, vec![seat.clone(), remote.to_string(), session_type.to_string(), class.to_string()])));
                }
            }
        }
    }

    Ok(())
}

fn known_string_or_other(value: &str, known: &[&str]) -> String {
    if known.contains(&value) {
        value.to_string()
    } else {
        "other".to_string()
    }
}