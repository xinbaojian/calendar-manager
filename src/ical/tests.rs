//! iCal 生成器测试
//! 验证 iCal 文件生成和时区处理

use crate::ical::ICalGenerator;
use crate::models::Event;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event() -> Event {
        Event {
            id: "evt_test_001".to_string(),
            user_id: "usr_test_001".to_string(),
            title: "测试事件".to_string(),
            description: Some("这是一个测试事件".to_string()),
            location: Some("会议室 A".to_string()),
            start_time: "2026-04-05T21:30:00+08:00".to_string(),
            end_time: "2026-04-05T22:30:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: Some(15),
            tags: Some("[\"工作\"]".to_string()),
            status: "active".to_string(),
            created_at: "2026-04-01T10:00:00+08:00".to_string(),
            updated_at: "2026-04-01T10:00:00+08:00".to_string(),
        }
    }

    #[test]
    fn test_ical_contains_timezone_component() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证包含 VTIMEZONE 组件
        assert!(ical.contains("BEGIN:VTIMEZONE"), "iCal should contain VTIMEZONE component");
        assert!(ical.contains("TZID:Asia/Shanghai"), "iCal should have Asia/Shanghai timezone");
        assert!(ical.contains("END:VTIMEZONE"), "iCal should close VTIMEZONE component");
    }

    #[test]
    fn test_ical_has_both_standard_and_daylight() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证包含标准时间和夏令时组件
        assert!(ical.contains("BEGIN:STANDARD"), "iCal should contain STANDARD component");
        assert!(ical.contains("BEGIN:DAYLIGHT"), "iCal should contain DAYLIGHT component");
        assert!(ical.contains("TZNAME:CST"), "iCal should have CST timezone name");
    }

    #[test]
    fn test_ical_datetime_has_tzid() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证 DTSTART 和 DTEND 使用 TZID 参数
        assert!(ical.contains("DTSTART;TZID=Asia/Shanghai:"), "DTSTART should have TZID parameter");
        assert!(ical.contains("DTEND;TZID=Asia/Shanghai:"), "DTEND should have TZID parameter");
    }

    #[test]
    fn test_ical_datetime_no_utc_suffix() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证时间格式不包含 Z 后缀（Z 表示 UTC）
        // 应该使用本地时间格式配合 TZID
        assert!(ical.contains("DTSTART;TZID=Asia/Shanghai:"), "Should find DTSTART with TZID");

        // 检查时间格式中紧跟日期时间后没有 Z（需要更精确的检查）
        let lines: Vec<&str> = ical.lines().collect();
        for line in lines {
            if line.starts_with("DTSTART;TZID=") || line.starts_with("DTEND;TZID=") {
                // 这些行应该以数字结尾，不是 Z
                // 排除空格（行折叠的情况）
                let trimmed = line.trim();
                assert!(!trimmed.ends_with('Z'), "DateTime with TZID should not have Z suffix: {}", line);
            }
        }
    }

    #[test]
    fn test_ical_contains_calendar_metadata() {
        let event = create_test_event();
        let calendar_name = "测试日历";
        let ical = ICalGenerator::generate(&[event], calendar_name);

        // 验证日历元数据
        assert!(ical.contains("BEGIN:VCALENDAR"), "iCal should start with VCALENDAR");
        assert!(ical.contains("END:VCALENDAR"), "iCal should end with VCALENDAR");
        assert!(ical.contains("VERSION:2.0"), "iCal should have VERSION:2.0");
        assert!(ical.contains("PRODID:"), "iCal should have PRODID");
        assert!(ical.contains("X-WR-CALNAME:测试日历"), "iCal should have calendar name");
        assert!(ical.contains("X-WR-TIMEZONE:Asia/Shanghai"), "iCal should declare timezone");
    }

    #[test]
    fn test_ical_contains_event_details() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证事件详情
        assert!(ical.contains("BEGIN:VEVENT"), "iCal should have VEVENT");
        assert!(ical.contains("SUMMARY:测试事件"), "iCal should have event summary");
        assert!(ical.contains("DESCRIPTION:这是一个测试事件"), "iCal should have description");
        assert!(ical.contains("LOCATION:会议室 A"), "iCal should have location");
        assert!(ical.contains("STATUS:CONFIRMED"), "iCal should have confirmed status");
    }

    #[test]
    fn test_ical_with_reminder() {
        let event = create_test_event();
        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证提醒组件
        assert!(ical.contains("BEGIN:VALARM"), "iCal should have VALARM component");
        assert!(ical.contains("TRIGGER:-PT15M"), "VALARM should have 15 minute trigger");
        assert!(ical.contains("ACTION:DISPLAY"), "VALARM should be DISPLAY action");
        assert!(ical.contains("END:VALARM"), "VALARM should be closed");
    }

    #[test]
    fn test_ical_skips_inactive_events() {
        let mut event = create_test_event();
        event.status = "cancelled".to_string();

        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 取消的事件不应该出现在 iCal 中
        assert!(!ical.contains("BEGIN:VEVENT"), "Cancelled events should not be in iCal");
    }

    #[test]
    fn test_ical_escape_special_characters() {
        let mut event = create_test_event();
        event.title = "测试;事件\\,带特殊字符".to_string();
        event.description = Some("描述\n带\n换行".to_string());

        let ical = ICalGenerator::generate(&[event], "测试日历");

        // 验证特殊字符被正确转义
        assert!(ical.contains("\\;"), "Semicolons should be escaped");
        assert!(ical.contains("\\,"), "Commas should be escaped");
        assert!(ical.contains("\\n"), "Newlines should be escaped");
    }

    #[test]
    fn test_ical_line_folding() {
        // 创建一个很长的标题来测试行折叠
        let mut event = create_test_event();
        event.title = "这是一个非常非常非常非常非常非常非常非常非常非常长的标题用来测试iCal规范的行折叠功能是否正常工作因为RFC5545规定每行不应该超过75个八位组".to_string();

        let ical = ICalGenerator::generate(&[event], "测试日历");

        // RFC 5545 规定行长度不超过 75 字节
        // 检查所有行都符合此规则
        for line in ical.lines() {
            // 排除续行（以空格开头的行）
            let check_line = if line.starts_with(' ') || line.starts_with("\t") {
                &line[1..]
            } else {
                line
            };

            // 行长度（字节）不应超过 75
            // UTF-8 编码的中文字符占用多个字节
            assert!(check_line.len() <= 75 || check_line.as_bytes().len() <= 75,
                    "Line should not exceed 75 bytes: {} ({} bytes)", check_line, check_line.as_bytes().len());
        }
    }

    #[test]
    fn test_ical_multiple_events() {
        let event1 = create_test_event();
        let mut event2 = create_test_event();
        event2.id = "evt_test_002".to_string();
        event2.title = "第二个事件".to_string();
        event2.start_time = "2026-04-06T10:00:00+08:00".to_string();
        event2.end_time = "2026-04-06T11:00:00+08:00".to_string();

        let ical = ICalGenerator::generate(&[event1, event2], "测试日历");

        // 验证两个事件都在 iCal 中
        assert!(ical.contains("SUMMARY:测试事件"), "First event should be present");
        assert!(ical.contains("SUMMARY:第二个事件"), "Second event should be present");

        // 计算事件数量（每个事件有一个 BEGIN:VEVENT）
        let event_count = ical.matches("BEGIN:VEVENT").count();
        assert_eq!(event_count, 2, "iCal should contain 2 events");
    }

    #[test]
    fn test_ical_unique_uid() {
        let event1 = create_test_event();
        let mut event2 = create_test_event();
        event2.id = "evt_test_002".to_string();

        let ical = ICalGenerator::generate(&[event1, event2], "测试日历");

        // 验证每个事件有唯一的 UID
        assert!(ical.contains("UID:evt_test_001@calendarsync"), "First event should have correct UID");
        assert!(ical.contains("UID:evt_test_002@calendarsync"), "Second event should have correct UID");
    }
}
