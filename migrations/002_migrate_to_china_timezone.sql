-- migrations/002_migrate_to_china_timezone.sql
-- 将所有 UTC 时间戳转换为东八区时间（+8 小时）
-- RFC3339 格式字符串可以直接进行字符串操作

-- 更新 events 表的时间字段
UPDATE events
SET
    start_time = CASE
        -- 如果已经是带时区的 RFC3339（如 2024-01-01T12:00:00+08:00），保持不变
        WHEN start_time LIKE '%+%' OR start_time LIKE '%Z' THEN start_time
        -- 否则认为是 UTC 时间，添加 +08:00 时区偏移
        ELSE substr(start_time, 1, 19) || '+08:00'
    END,
    end_time = CASE
        WHEN end_time LIKE '%+%' OR end_time LIKE '%Z' THEN end_time
        ELSE substr(end_time, 1, 19) || '+08:00'
    END,
    created_at = CASE
        WHEN created_at LIKE '%+%' OR created_at LIKE '%Z' THEN created_at
        ELSE substr(created_at, 1, 19) || '+08:00'
    END,
    updated_at = CASE
        WHEN updated_at LIKE '%+%' OR updated_at LIKE '%Z' THEN updated_at
        ELSE substr(updated_at, 1, 19) || '+08:00'
    END
WHERE
    -- 只处理没有时区信息的记录（不带 + 且不带 Z）
    start_time NOT LIKE '%+%' AND start_time NOT LIKE '%Z';

-- 更新 users 表的时间字段
UPDATE users
SET
    created_at = CASE
        WHEN created_at LIKE '%+%' OR created_at LIKE '%Z' THEN created_at
        ELSE substr(created_at, 1, 19) || '+08:00'
    END,
    updated_at = CASE
        WHEN updated_at LIKE '%+%' OR updated_at LIKE '%Z' THEN updated_at
        ELSE substr(updated_at, 1, 19) || '+08:00'
    END
WHERE
    created_at NOT LIKE '%+%' AND created_at NOT LIKE '%Z';

-- 更新 webhooks 表的时间字段
UPDATE webhooks
SET
    created_at = CASE
        WHEN created_at LIKE '%+%' OR created_at LIKE '%Z' THEN created_at
        ELSE substr(created_at, 1, 19) || '+08:00'
    END
WHERE
    created_at NOT LIKE '%+%' AND created_at NOT LIKE '%Z';

-- 更新 webhook_logs 表的时间字段
UPDATE webhook_logs
SET
    sent_at = CASE
        WHEN sent_at LIKE '%+%' OR sent_at LIKE '%Z' THEN sent_at
        ELSE substr(sent_at, 1, 19) || '+08:00'
    END
WHERE
    sent_at NOT LIKE '%+%' AND sent_at NOT LIKE '%Z';

-- 添加注释说明迁移已完成
-- 执行后，所有时间戳都应该是东八区时间（+08:00）
