use chrono::{DateTime, Utc};
use chrono_tz::Asia::Shanghai;

use crate::models::Event;

pub struct ICalGenerator;

impl ICalGenerator {
    pub fn generate(events: &[Event], calendar_name: &str) -> String {
        let mut ical = String::new();
        let dtstamp = Utc::now().with_timezone(&Shanghai).format("%Y%m%dT%H%M%S");

        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//CalendarSync//CN\r\n");
        ical.push_str("CALSCALE:GREGORIAN\r\n");
        ical.push_str("METHOD:PUBLISH\r\n");
        ical.push_str(&fold_line(&format!(
            "X-WR-CALNAME:{}",
            escape_ical_text(calendar_name)
        )));
        ical.push_str(&fold_line("X-WR-CALDESC:CalendarSync 日程订阅"));
        ical.push_str("X-WR-TIMEZONE:Asia/Shanghai\r\n");

        // 添加 VTIMEZONE 组件定义 Asia/Shanghai 时区
        // 中国标准时间（CST）不使用夏令时，所有时间都是 UTC+8
        ical.push_str("BEGIN:VTIMEZONE\r\n");
        ical.push_str("TZID:Asia/Shanghai\r\n");
        ical.push_str("X-LIC-LOCATION:Asia/Shanghai\r\n");
        // 标准时间组件（全年适用）
        ical.push_str("BEGIN:STANDARD\r\n");
        ical.push_str("DTSTART:19700101T000000\r\n");
        ical.push_str("TZOFFSETFROM:+0800\r\n");
        ical.push_str("TZOFFSETTO:+0800\r\n");
        ical.push_str("TZNAME:CST\r\n");
        ical.push_str("END:STANDARD\r\n");
        // 中国不使用夏令时，但某些日历客户端可能期望此组件
        // 因此添加一个空的 DAYLIGHT 组件，与标准时间相同
        ical.push_str("BEGIN:DAYLIGHT\r\n");
        ical.push_str("DTSTART:19700101T000000\r\n");
        ical.push_str("TZOFFSETFROM:+0800\r\n");
        ical.push_str("TZOFFSETTO:+0800\r\n");
        ical.push_str("TZNAME:CST\r\n");
        ical.push_str("END:DAYLIGHT\r\n");
        ical.push_str("END:VTIMEZONE\r\n");

        for event in events {
            if event.status != "active" {
                continue;
            }

            ical.push_str("BEGIN:VEVENT\r\n");

            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.start_time) {
                let local_dt = dt.with_timezone(&Shanghai);
                ical.push_str(&fold_line(&format!(
                    "DTSTART;TZID=Asia/Shanghai:{}",
                    local_dt.format("%Y%m%dT%H%M%S")
                )));
            }
            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.end_time) {
                let local_dt = dt.with_timezone(&Shanghai);
                ical.push_str(&fold_line(&format!(
                    "DTEND;TZID=Asia/Shanghai:{}",
                    local_dt.format("%Y%m%dT%H%M%S")
                )));
            }

            ical.push_str(&fold_line(&format!("DTSTAMP:{dtstamp}")));
            ical.push_str(&fold_line(&format!("UID:{}@calendarsync", event.id)));
            ical.push_str(&fold_line(&format!(
                "SUMMARY:{}",
                escape_ical_text(&event.title)
            )));

            if let Some(desc) = &event.description {
                ical.push_str(&fold_line(&format!(
                    "DESCRIPTION:{}",
                    escape_ical_text(desc)
                )));
            }

            if let Some(location) = &event.location {
                ical.push_str(&fold_line(&format!(
                    "LOCATION:{}",
                    escape_ical_text(location)
                )));
            }

            ical.push_str("STATUS:CONFIRMED\r\n");

            if let Some(minutes) = event.reminder_minutes {
                ical.push_str("BEGIN:VALARM\r\n");
                ical.push_str(&fold_line(&format!("TRIGGER:-PT{}M", minutes)));
                ical.push_str("ACTION:DISPLAY\r\n");
                ical.push_str(&fold_line("DESCRIPTION:日程提醒"));
                ical.push_str("END:VALARM\r\n");
            }

            ical.push_str("END:VEVENT\r\n");
        }

        ical.push_str("END:VCALENDAR\r\n");
        ical
    }
}

/// Fold lines longer than 75 octets per RFC 5545
fn fold_line(line: &str) -> String {
    const MAX_LINE_LENGTH: usize = 75;

    let bytes = line.as_bytes();
    if bytes.len() <= MAX_LINE_LENGTH {
        return format!("{line}\r\n");
    }

    let mut result = String::new();
    let mut pos = 0;
    let mut first = true;

    while pos < bytes.len() {
        // Find a safe UTF-8 boundary at or before MAX_LINE_LENGTH
        let end = (pos + MAX_LINE_LENGTH).min(bytes.len());
        // Ensure we don't split a multi-byte character
        let safe_end = if end < bytes.len() {
            let mut split = end;
            while split > pos && (bytes[split] & 0xC0) == 0x80 {
                split -= 1;
            }
            if split == pos {
                // Single byte takes more than MAX_LINE_LENGTH, force split
                end
            } else {
                split
            }
        } else {
            end
        };

        if first {
            result.push_str(&line[pos..safe_end]);
            result.push_str("\r\n ");
            first = false;
        } else {
            result.push_str(&line[pos..safe_end]);
            if safe_end < bytes.len() {
                result.push_str("\r\n ");
            }
        }
        pos = safe_end;
    }

    result.push_str("\r\n");
    result
}

fn escape_ical_text(text: &str) -> String {
    text.replace('\r', "")
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}
