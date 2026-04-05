//! 时区相关测试
//! 验证东八区（Asia/Shanghai）时间处理是否正确

use chrono::{Utc, TimeZone, Datelike, Timelike, Offset};
use chrono_tz::Asia::Shanghai;

#[test]
fn test_shanghai_timezone_offset() {
    // 验证上海时区偏移为 UTC+8
    let now_utc = Utc::now();
    let now_shanghai = now_utc.with_timezone(&Shanghai);

    // 计算时差（应该是 8 小时）
    let offset = now_shanghai.offset().fix().local_minus_utc();
    assert_eq!(offset, 8 * 3600, "Shanghai timezone should be UTC+8");
}

#[test]
fn test_rfc3339_format_with_timezone() {
    // 测试带时区的 RFC3339 格式化
    let time = Shanghai.with_ymd_and_hms(2026, 4, 5, 21, 30, 0).unwrap();
    let rfc3339 = time.to_rfc3339();

    // 验证格式正确
    assert!(rfc3339.contains("+08:00"), "RFC3339 should contain +08:00 timezone offset");
    assert!(rfc3339.starts_with("2026-04-05T21:30:00"), "RFC3339 should start with correct datetime");
}

#[test]
fn test_timezone_conversion_preserves_local_time() {
    // 测试时区转换保持本地时间不变
    let shanghai_time = Shanghai.with_ymd_and_hms(2026, 4, 5, 21, 30, 0).unwrap();
    let utc_time = shanghai_time.to_utc();

    // 转换回上海时区应该得到相同的时间
    let converted_back = utc_time.with_timezone(&Shanghai);
    assert_eq!(shanghai_time, converted_back, "Round-trip conversion should preserve time");
}

#[test]
fn test_rfc3339_string_comparison() {
    // 测试 RFC3339 字符串比较的正确性
    // RFC3339 字符串的字典序与时间序一致

    let time1 = "2026-04-05T21:30:00+08:00";
    let time2 = "2026-04-05T22:30:00+08:00";
    let time3 = "2026-04-05T21:30:00+08:00";

    assert!(time1 < time2, "Earlier time should be less in string comparison");
    assert_eq!(time1, time3, "Equal times should be equal in string comparison");
}

#[test]
fn test_utc_to_shanghai_conversion() {
    // 测试 UTC 到上海时区的转换
    let utc_time = Utc.with_ymd_and_hms(2026, 4, 5, 13, 30, 0).unwrap();
    let shanghai_time = utc_time.with_timezone(&Shanghai);

    // UTC 13:30 应该对应上海时间 21:30（相差 8 小时）
    assert_eq!(shanghai_time.hour(), 21);
    assert_eq!(shanghai_time.minute(), 30);
}

#[test]
fn test_duration_operations_with_timezone() {
    // 测试带时区的时间计算
    let base_time = Shanghai.with_ymd_and_hms(2026, 4, 5, 12, 0, 0).unwrap();
    let future_time = base_time + chrono::Duration::days(7);

    assert_eq!(future_time.day(), 12);
    assert_eq!(future_time.month(), 4);
}

#[test]
fn test_parse_rfc3339_with_timezone() {
    // 测试解析带时区的 RFC3339 字符串
    let rfc_time = "2026-04-05T21:30:00+08:00";
    let parsed = chrono::DateTime::parse_from_rfc3339(rfc_time)
        .expect("Should parse valid RFC3339");

    // 验证解析结果
    assert_eq!(parsed.year(), 2026);
    assert_eq!(parsed.month(), 4);
    assert_eq!(parsed.day(), 5);
    assert_eq!(parsed.hour(), 21);
    assert_eq!(parsed.minute(), 30);
}

#[test]
fn test_timezone_aware_formatting() {
    // 测试时区感知的格式化输出
    let shanghai_time = Shanghai.with_ymd_and_hms(2026, 4, 5, 21, 30, 0).unwrap();

    // RFC3339 格式
    let rfc3339 = shanghai_time.to_rfc3339();
    assert!(rfc3339.ends_with("+08:00"));

    // 自定义格式（iCal 使用）
    let ical_format = shanghai_time.format("%Y%m%dT%H%M%S").to_string();
    assert_eq!(ical_format, "20260405T213000");
}

// ========== iCal 集成测试 ==========

/// 测试生成的事件时间字符串在 iCal 中的格式
#[test]
fn test_ical_datetime_format() {
    let shanghai_time = Shanghai.with_ymd_and_hms(2026, 4, 5, 21, 30, 0).unwrap();

    // iCal 格式应该是 YYYYMMDDTHHMMSS（不带 Z）
    let ical_format = shanghai_time.format("%Y%m%dT%H%M%S").to_string();

    // 验证格式符合 iCal 标准
    assert!(!ical_format.contains('Z'), "iCal local time should not have Z suffix");
    assert_eq!(ical_format.len(), 15, "iCal datetime should be exactly 15 characters");
}

/// 测试边界情况：月末日期
#[test]
fn test_month_boundary() {
    // 1月31日 + 1天 = 2月1日
    let jan_31 = Shanghai.with_ymd_and_hms(2026, 1, 31, 12, 0, 0).unwrap();
    let feb_1 = jan_31 + chrono::Duration::days(1);

    assert_eq!(feb_1.month(), 2);
    assert_eq!(feb_1.day(), 1);
}

/// 测试闰年日期
#[test]
fn test_leap_year() {
    // 2024是闰年，2月有29天
    let feb_28_2024 = Shanghai.with_ymd_and_hms(2024, 2, 28, 0, 0, 0).unwrap();
    let feb_29_2024 = feb_28_2024 + chrono::Duration::days(1);

    assert_eq!(feb_29_2024.month(), 2);
    assert_eq!(feb_29_2024.day(), 29);
}

/// 测试跨天事件
#[test]
fn test_midnight_crossing() {
    // 23:00 + 2小时 = 次日 01:00
    let time = Shanghai.with_ymd_and_hms(2026, 4, 5, 23, 0, 0).unwrap();
    let next_day = time + chrono::Duration::hours(2);

    assert_eq!(next_day.day(), 6);
    assert_eq!(next_day.hour(), 1);
}

