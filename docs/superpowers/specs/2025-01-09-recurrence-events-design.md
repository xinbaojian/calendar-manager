# 重复日程功能设计文档

**日期:** 2025-01-09
**状态:** 已批准
**作者:** Claude Code

## 规格审查

**审查日期:** 2025-01-09
**审查状态:** ✅ 批准
**审查人:** Spec Document Reviewer

**审查结果:**

规格文档已完成并批准实施。文档提供了：

1. **明确的功能需求** - 具体的重复类型、结束条件和UI需求
2. **完整的技术设计** - 数据流程、组件、数据库架构（无需修改）、RRULE格式规范
3. **详细的前端设计** - HTML结构、RRULE生成器和实例计算器的JavaScript类
4. **后端设计** - iCal生成器更新、API考量
5. **分阶段实施计划** - 三个阶段，每个阶段都有明确的交付物
6. **技术约束和安全** - 时区处理、性能限制、安全考虑

规格文档充分利用了现有的数据库字段和API端点，显示了对当前系统架构的良好理解。分阶段方法允许增量交付和验证。

**建议（非阻塞）:**
- 可以考虑添加明确的错误处理需求（例如：RRULE解析失败时的处理）
- 可以添加更具体的性能指标，而不是"1000个实例内秒级响应" - 是1秒还是5秒？
- 可选的API端点设计可以在作为初始实施的一部分时更完整地指定

这些建议不影响规格文档进入实施计划的就绪状态。

## 概述

为 CalendarSync 添加重复日程功能，支持用户创建按周期重复的日程，并能通过 iCal (.ics) 格式订阅到 iPhone/macOS 日历中。

## 需求

### 功能需求

1. **重复类型支持**
   - 基础重复：每天、每周、每月、每年
   - 自定义间隔：每 N 天/周/月/年
   - 高级规则：指定星期几、每月第几天等

2. **结束条件**
   - 按日期结束：指定结束日期
   - 按次数结束：重复 N 次后停止
   - 永不结束：无限重复

3. **用户界面**
   - 快速预设按钮（日/周/月/年）
   - 高级自定义表单
   - 混合方案交互

4. **显示方式**
   - 列表默认显示原始日程
   - 支持按需展开查看所有实例

### 非功能需求

1. 遵循 iCal RFC 5545 标准
2. 与现有数据库结构兼容
3. 前端性能：支持快速展开大量实例
4. iCal 兼容性：确保 iPhone/macOS 日历正确解析

## 架构设计

### 数据流程

```
用户设置重复规则
    ↓
前端生成 RRULE 字符串
    ↓
提交到后端 API
    ↓
存储到数据库 (recurrence_rule 字段)
    ↓
iCal 导出时包含 RRULE
    ↓
客户端日历解析并显示重复实例
```

### 核心组件

1. **RRULEGenerator (前端)** - 生成 iCal RRULE 格式
2. **RRULEParser (前端/后端)** - 解析 RRULE 计算实例
3. **RecurrenceEditor (前端)** - 重复规则编辑UI组件
4. **ICalGenerator (后端)** - 更新以支持 RRULE 输出

## 数据模型

### 数据库

**无需修改** - 现有字段已足够：

```sql
CREATE TABLE events (
    ...
    recurrence_rule TEXT,        -- 存储 RRULE 字符串
    recurrence_until TEXT,       -- 便捷查询字段
    ...
);
```

### Event 模型

```rust
pub struct Event {
    pub id: String,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,    // 新增使用
    pub recurrence_until: Option<String>,   // 新增使用
    ...
}
```

## RRULE 格式规范

使用标准 iCal RRULE 格式 (RFC 5545)

### 基础示例

| 用户选择 | RRULE 输出 |
|---------|-----------|
| 每天重复 | `FREQ=DAILY` |
| 每周重复 | `FREQ=WEEKLY` |
| 每月重复 | `FREQ=MONTHLY` |
| 每年重复 | `FREQ=YEARLY` |
| 每2周重复 | `FREQ=WEEKLY;INTERVAL=2` |
| 每周一、三 | `FREQ=WEEKLY;BYDAY=MO,WE` |
| 每月第一个周一 | `FREQ=MONTHLY;BYDAY=1MO` |

### 结束条件

| 条件 | RRULE 输出 |
|-----|-----------|
| 永不结束 | `FREQ=DAILY` |
| 2025-12-31结束 | `FREQ=DAILY;UNTIL=20251231T235959Z` |
| 重复10次 | `FREQ=DAILY;COUNT=10` |

## 前端设计

### UI 组件

```html
<!-- 重复设置区域 -->
<div class="recurrence-editor">
  <!-- 快速预设 -->
  <div class="recurrence-presets">
    <button data-freq="none">不重复</button>
    <button data-freq="daily">每天</button>
    <button data-freq="weekly">每周</button>
    <button data-freq="monthly">每月</button>
    <button data-freq="yearly">每年</button>
    <button data-freq="custom">自定义...</button>
  </div>

  <!-- 自定义表单 (默认隐藏) -->
  <div class="recurrence-custom" style="display:none;">
    <select id="recurrenceFreq">
      <option value="DAILY">日</option>
      <option value="WEEKLY">周</option>
      <option value="MONTHLY">月</option>
      <option value="YEARLY">年</option>
    </select>
    <input type="number" id="recurrenceInterval" value="1" min="1">

    <!-- 周选择 (仅当FREQ=WEEKLY时显示) -->
    <div id="bydaySelector">
      <label><input type="checkbox" value="MO">周一</label>
      <label><input type="checkbox" value="WE">周三</label>
      <label><input type="checkbox" value="FR">周五</label>
    </div>

    <!-- 结束条件 -->
    <div class="recurrence-end">
      <select id="endType">
        <option value="never">永不</option>
        <option value="date">按日期</option>
        <option value="count">按次数</option>
      </select>
      <input type="date" id="endDate">
      <input type="number" id="endCount" min="1" max="999">
    </div>
  </div>
</div>
```

### RRULE 生成器

```javascript
class RecurrenceRuleGenerator {
  generate(options) {
    const { freq, interval = 1, byday = [], bymonthday = null, endType, endDate, endCount } = options;

    let rrule = `FREQ=${freq}`;
    if (interval > 1) rrule += `;INTERVAL=${interval}`;
    if (byday.length) rrule += `;BYDAY=${byday.join(',')}`;
    if (bymonthday) rrule += `;BYMONTHDAY=${bymonthday}`;

    if (endType === 'date' && endDate) {
      rrule += `;UNTIL=${this.formatUntil(endDate)}`;
    } else if (endType === 'count' && endCount) {
      rrule += `;COUNT=${endCount}`;
    }

    return rrule;
  }

  parse(rrule) {
    // 解析 RRULE 字符串用于编辑
    // 返回 { freq, interval, byday, endType, ... }
  }

  formatUntil(date) {
    // 转换为 UTC 格式: YYYYMMDDTHHMMSSZ
  }
}
```

### 实例计算器

```javascript
class RecurrenceInstanceCalculator {
  getInstances(event, startDate, endDate) {
    if (!event.recurrence_rule) return [event];

    const rule = this.parseRRULE(event.recurrence_rule);
    const instances = [];

    let current = new Date(event.start_time);
    let count = 0;

    while (current <= endDate) {
      if (current >= startDate) {
        instances.push({
          ...event,
          start_time: current.toISOString(),
          is_instance: true
        });
      }

      current = this.nextOccurrence(current, rule);
      count++;

      if (rule.count && count >= rule.count) break;
      if (rule.until && current > rule.until) break;
      if (count > 1000) break; // 安全限制
    }

    return instances;
  }
}
```

## 后端设计

### iCal 生成器更新

```rust
// src/ical/generator.rs
impl ICalGenerator {
    pub fn generate(events: &[Event], calendar_name: &str) -> String {
        let mut ical = String::new();
        // ... header ...

        for event in events {
            if event.status != "active" { continue; }

            ical.push_str("BEGIN:VEVENT\r\n");
            // ... existing fields (DTSTART, DTEND, SUMMARY, etc.) ...

            // 添加 RRULE
            if let Some(ref rrule) = event.recurrence_rule {
                ical.push_str(&fold_line(&format!("RRULE:{}", rrule)));
            }

            ical.push_str("END:VEVENT\r\n");
        }

        ical.push_str("END:VCALENDAR\r\n");
        ical
    }
}
```

### API 变更

**现有API无需修改** - `/api/events` POST/PUT 已支持 `recurrence_rule` 字段

**可选新增API：**

```
GET /api/events/:id/instances?from=2025-01-01&to=2025-12-31
```

返回指定时间范围内重复事件的所有实例。

## 实现计划

### 第一阶段：基础功能

1. **前端UI**
   - [ ] 添加重复设置组件到事件表单
   - [ ] 实现RRULE生成器
   - [ ] 更新表单提交逻辑

2. **后端**
   - [ ] 更新iCal生成器支持RRULE
   - [ ] 测试iCal输出

### 第二阶段：增强功能

1. **前端**
   - [ ] 实现RRULE解析器
   - [ ] 添加实例展开功能
   - [ ] 支持编辑重复规则

2. **后端（可选）**
   - [ ] 添加实例查询API
   - [ ] 性能优化

### 第三阶段：测试

1. **单元测试**
   - [ ] RRULE生成器测试
   - [ ] RRULE解析器测试
   - [ ] iCal生成器测试

2. **集成测试**
   - [ ] 创建重复事件
   - [ ] 编辑重复事件
   - [ ] 删除重复事件
   - [ ] iCal订阅验证

3. **手动测试**
   - [ ] iPhone日历订阅
   - [ ] macOS日历订阅
   - [ ] 边界情况测试

## 技术约束

1. **时区处理**
   - 所有时间使用 Asia/Shanghai 时区
   - UNTIL 日期需转换为正确时区

2. **性能考虑**
   - 限制计算实例数量上限
   - 避免前端计算过多实例

3. **兼容性**
   - 确保 RRULE 符合 RFC 5545
   - 测试 Apple 日历兼容性

## 安全考虑

1. 防止恶意构造的 RRULE 导致无限循环
2. 限制用户可创建的重复事件数量
3. 实例计算设置合理的超时和数量限制

## 成功标准

1. ✅ 用户可通过UI创建重复日程
2. ✅ 重复日程在iCal中正确显示
3. ✅ iPhone/macOS日历正确解析重复规则
4. ✅ 支持编辑和删除重复日程
5. ✅ 性能满足：1000个实例以内秒级响应
