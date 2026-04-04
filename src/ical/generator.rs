use chrono::{DateTime, Utc};

use crate::models::Event;

pub struct ICalGenerator;

impl ICalGenerator {
    pub fn generate(events: &[Event], calendar_name: &str) -> String {
        let mut ical = String::new();

        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//CalendarSync//CN\r\n");
        ical.push_str("CALSCALE:GREGORIAN\r\n");
        ical.push_str("METHOD:PUBLISH\r\n");
        ical.push_str(&format!("X-WR-CALNAME:{}\r\n", calendar_name));
        ical.push_str("X-WR-CALDESC:CalendarSync 日程订阅\r\n");

        for event in events {
            if event.status != "active" {
                continue;
            }

            ical.push_str("BEGIN:VEVENT\r\n");

            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.start_time) {
                ical.push_str(&format!(
                    "DTSTART:{}\r\n",
                    dt.format("%Y%m%dT%H%M%SZ")
                ));
            }
            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.end_time) {
                ical.push_str(&format!(
                    "DTEND:{}\r\n",
                    dt.format("%Y%m%dT%H%M%SZ")
                ));
            }

            ical.push_str(&format!(
                "DTSTAMP:{}\r\n",
                Utc::now().format("%Y%m%dT%H%M%SZ")
            ));
            ical.push_str(&format!(
                "UID:{}@calendarsync\r\n",
                event.id
            ));
            ical.push_str(&format!(
                "SUMMARY:{}\r\n",
                escape_ical_text(&event.title)
            ));

            if let Some(desc) = &event.description {
                ical.push_str(&format!(
                    "DESCRIPTION:{}\r\n",
                    escape_ical_text(desc)
                ));
            }

            if let Some(location) = &event.location {
                ical.push_str(&format!(
                    "LOCATION:{}\r\n",
                    escape_ical_text(location)
                ));
            }

            ical.push_str("STATUS:CONFIRMED\r\n");

            if let Some(minutes) = event.reminder_minutes {
                ical.push_str("BEGIN:VALARM\r\n");
                ical.push_str(&format!("TRIGGER:-PT{}M\r\n", minutes));
                ical.push_str("ACTION:DISPLAY\r\n");
                ical.push_str("DESCRIPTION:日程提醒\r\n");
                ical.push_str("END:VALARM\r\n");
            }

            ical.push_str("END:VEVENT\r\n");
        }

        ical.push_str("END:VCALENDAR\r\n");
        ical
    }
}

fn escape_ical_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}
