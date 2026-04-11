-- migrations/003_normalize_event_timestamps.sql
-- 将 events 表中所有时间字段归一化为上海时区（+08:00）格式
-- 解决前端 toISOString() 存储的 UTC 时间（Z 后缀）与后端上海时区字符串比较不一致的问题

-- 处理 end_time
-- 对于非 +08:00 格式的时间，提取前19字符（YYYY-MM-DDTHH:MM:SS），加8小时后追加 +08:00
UPDATE events
SET end_time = strftime('%Y-%m-%dT%H:%M:%S+08:00', substr(end_time, 1, 19), '+8 hours')
WHERE end_time NOT LIKE '%+08:00';

UPDATE events
SET start_time = strftime('%Y-%m-%dT%H:%M:%S+08:00', substr(start_time, 1, 19), '+8 hours')
WHERE start_time NOT LIKE '%+08:00';

UPDATE events
SET recurrence_until = strftime('%Y-%m-%dT%H:%M:%S+08:00', substr(recurrence_until, 1, 19), '+8 hours')
WHERE recurrence_until IS NOT NULL AND recurrence_until NOT LIKE '%+08:00';
