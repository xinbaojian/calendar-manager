use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 创建日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateEventParams {
    #[schemars(description = "日程标题")]
    pub title: String,

    #[schemars(description = "日程描述")]
    pub description: Option<String>,

    #[schemars(description = "地点")]
    pub location: Option<String>,

    #[schemars(description = "开始时间 (RFC3339格式，上海时区)")]
    pub start_time: String,

    #[schemars(description = "结束时间 (RFC3339格式，上海时区)")]
    pub end_time: String,

    #[schemars(description = "重复规则 (RRULE格式)")]
    pub recurrence_rule: Option<String>,

    #[schemars(description = "重复结束时间 (RFC3339格式)")]
    pub recurrence_until: Option<String>,

    #[schemars(description = "提前提醒分钟数")]
    pub reminder_minutes: Option<i32>,

    #[schemars(description = "标签")]
    pub tags: Option<Vec<String>>,
}

/// 查询日程列表参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListEventsParams {
    #[schemars(description = "开始日期过滤 (RFC3339格式)")]
    pub from: Option<String>,

    #[schemars(description = "结束日期过滤 (RFC3339格式)")]
    pub to: Option<String>,

    #[schemars(description = "状态过滤 (active, expired, all)")]
    pub status: Option<String>,

    #[schemars(description = "关键词搜索")]
    pub keyword: Option<String>,
}

/// 获取单个日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,
}

/// 更新日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,

    #[schemars(description = "日程标题")]
    pub title: Option<String>,

    #[schemars(description = "日程描述")]
    pub description: Option<String>,

    #[schemars(description = "地点")]
    pub location: Option<String>,

    #[schemars(description = "开始时间 (RFC3339格式，上海时区)")]
    pub start_time: Option<String>,

    #[schemars(description = "结束时间 (RFC3339格式，上海时区)")]
    pub end_time: Option<String>,

    #[schemars(description = "状态 (active, expired)")]
    pub status: Option<String>,

    #[schemars(description = "重复规则 (RRULE格式)")]
    pub recurrence_rule: Option<String>,

    #[schemars(description = "重复结束时间 (RFC3339格式)")]
    pub recurrence_until: Option<String>,

    #[schemars(description = "提前提醒分钟数")]
    pub reminder_minutes: Option<i32>,

    #[schemars(description = "标签")]
    pub tags: Option<Vec<String>>,
}

/// 删除日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DeleteEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,
}
