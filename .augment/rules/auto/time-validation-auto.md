# 时间验证自动规则

## 🔄 自动触发声明
**当检测到时间相关内容时，本规则自动触发执行时间验证。**

## 📋 触发条件

### 自动检测关键词
- 时间、日期、创建时间、更新时间
- timestamp、datetime、created、updated
- 年、月、日、时、分、秒
- 2025、2024、2023等年份数字

### 自动检测模式
```bash
# 检测时间相关内容的正则表达式
TIME_PATTERNS=(
    "时间|日期|创建|更新"
    "timestamp|datetime|created|updated"
    "20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]"
    "年|月|日|时|分|秒"
)
```

## 🔧 自动执行机制

### 检测到时间内容时自动执行
```bash
#!/bin/bash
# 时间验证自动脚本

echo "=== 自动时间验证触发 ==="

# 1. 获取当前实际时间
CURRENT_TIME=$(date '+%Y-%m-%d %H:%M:%S')
CURRENT_DATE=$(date '+%Y-%m-%d')

echo "当前系统时间: $CURRENT_TIME"

# 2. 检查是否使用了硬编码时间
echo "检查硬编码时间..."
if echo "$CONTENT" | grep -E "2025-01-|2024-12-|2023-"; then
    echo "❌ 检测到硬编码时间，违反规则"
    echo "正在替换为实际系统时间..."
    
    # 自动替换硬编码时间
    CONTENT=$(echo "$CONTENT" | sed "s/2025-01-[0-9][0-9]/$CURRENT_DATE/g")
    echo "✅ 已替换为实际时间: $CURRENT_DATE"
fi

# 3. 验证时间格式
echo "验证时间格式..."
if echo "$CONTENT" | grep -E "[0-9]{4}-[0-9]{2}-[0-9]{2}"; then
    echo "✅ 时间格式正确"
else
    echo "⚠️ 建议使用标准时间格式: YYYY-MM-DD"
fi

echo "=== 自动验证完成 ==="
```

### 文档生成时自动时间戳
```bash
# 自动为文档添加时间戳
add_timestamp() {
    local file="$1"
    local current_time=$(date '+%Y-%m-%d %H:%M:%S')
    local current_date=$(date '+%Y-%m-%d')
    
    # 在文档末尾自动添加时间戳
    cat >> "$file" << EOF

---
**创建时间**: $current_date
**最后更新**: $current_time
**自动生成**: 时间验证自动规则
EOF
    
    echo "✅ 已自动添加时间戳到 $file"
}
```

## 🔍 自动验证流程

### 内容生成前验证
```bash
# 在生成任何包含时间的内容前自动执行
pre_generation_check() {
    echo "=== 生成前时间检查 ==="
    
    # 确保系统时间可用
    if ! command -v date &> /dev/null; then
        echo "❌ date命令不可用"
        exit 1
    fi
    
    # 获取并显示当前时间
    local current_time=$(date '+%Y-%m-%d %H:%M:%S')
    echo "✅ 系统时间可用: $current_time"
    
    # 设置时间变量供后续使用
    export CURRENT_TIME="$current_time"
    export CURRENT_DATE=$(date '+%Y-%m-%d')
    
    echo "=== 检查完成 ==="
}
```

### 内容生成后验证
```bash
# 在生成内容后自动验证
post_generation_check() {
    local content="$1"
    echo "=== 生成后时间验证 ==="
    
    # 检查是否包含当前日期
    if echo "$content" | grep -q "$CURRENT_DATE"; then
        echo "✅ 使用了当前系统日期"
    else
        echo "⚠️ 未发现当前系统日期，请检查"
    fi
    
    # 检查是否包含硬编码时间
    if echo "$content" | grep -E "2025-01-|2024-12-"; then
        echo "❌ 发现硬编码时间，需要修正"
        return 1
    else
        echo "✅ 未发现硬编码时间"
    fi
    
    echo "=== 验证完成 ==="
}
```

## 🚨 自动修正机制

### 硬编码时间自动替换
```bash
# 自动替换硬编码时间
auto_fix_hardcoded_time() {
    local file="$1"
    local current_date=$(date '+%Y-%m-%d')
    local current_time=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo "=== 自动修正硬编码时间 ==="
    
    # 替换常见的硬编码时间模式
    sed -i "s/2025-01-[0-9][0-9]/$current_date/g" "$file"
    sed -i "s/2024-12-[0-9][0-9]/$current_date/g" "$file"
    sed -i "s/创建时间.*: 2025-01-[0-9][0-9]/创建时间: $current_date/g" "$file"
    sed -i "s/最后更新.*: 2025-01-[0-9][0-9]/最后更新: $current_time/g" "$file"
    
    echo "✅ 已自动修正硬编码时间"
    echo "当前时间: $current_time"
}
```

### 时间格式自动标准化
```bash
# 自动标准化时间格式
auto_standardize_time_format() {
    local file="$1"
    
    echo "=== 自动标准化时间格式 ==="
    
    # 标准化日期格式为 YYYY-MM-DD
    sed -i 's/[0-9]\{4\}\/[0-9]\{2\}\/[0-9]\{2\}/$(date -d "$0" "+%Y-%m-%d")/g' "$file"
    
    # 标准化时间格式为 YYYY-MM-DD HH:MM:SS
    sed -i 's/[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\} [0-9]\{2\}:[0-9]\{2\}/$(date -d "$0" "+%Y-%m-%d %H:%M:%S")/g' "$file"
    
    echo "✅ 已自动标准化时间格式"
}
```

## 🔄 集成机制

### 与Always规则集成
```bash
# 自动触发Always规则中的时间处理
trigger_always_time_rule() {
    echo "=== 触发Always时间规则 ==="
    
    # 调用Always规则中的时间处理机制
    source .augment/rules/always/01-time-handling-always.md
    
    # 执行强制时间检查
    CURRENT_TIME=$(date '+%Y-%m-%d %H:%M:%S')
    CURRENT_DATE=$(date '+%Y-%m-%d')
    
    echo "✅ Always时间规则已触发"
    echo "当前时间: $CURRENT_TIME"
}
```

### 与其他Auto规则协调
```bash
# 与文档格式规则协调
coordinate_with_document_format() {
    echo "=== 与文档格式规则协调 ==="
    
    # 确保时间戳格式符合文档格式要求
    local timestamp_format="**创建时间**: $(date '+%Y-%m-%d')"
    local update_format="**最后更新**: $(date '+%Y-%m-%d %H:%M:%S')"
    
    echo "标准时间戳格式:"
    echo "$timestamp_format"
    echo "$update_format"
    
    echo "✅ 时间格式已与文档格式协调"
}
```

## 📊 自动监控统计

### 触发次数统计
```bash
# 记录自动触发统计
log_trigger_stats() {
    local trigger_time=$(date '+%Y-%m-%d %H:%M:%S')
    local log_file=".augment/logs/time-validation-auto.log"
    
    mkdir -p .augment/logs
    echo "$trigger_time - 时间验证自动规则触发" >> "$log_file"
    
    # 统计今日触发次数
    local today=$(date '+%Y-%m-%d')
    local today_count=$(grep "$today" "$log_file" | wc -l)
    
    echo "今日触发次数: $today_count"
}
```

### 修正成功率统计
```bash
# 统计自动修正成功率
calculate_fix_success_rate() {
    local log_file=".augment/logs/time-validation-auto.log"
    
    if [ -f "$log_file" ]; then
        local total_triggers=$(wc -l < "$log_file")
        local successful_fixes=$(grep "修正成功" "$log_file" | wc -l)
        
        if [ $total_triggers -gt 0 ]; then
            local success_rate=$((successful_fixes * 100 / total_triggers))
            echo "自动修正成功率: $success_rate%"
        fi
    fi
}
```

## 🎯 优化建议

### 性能优化
- 缓存系统时间调用结果
- 批量处理多个文件的时间验证
- 异步执行非关键验证步骤

### 准确性提升
- 增加更多时间格式的识别模式
- 改进硬编码时间的检测算法
- 添加时区处理支持

---

**规则版本**: v1.0
**创建时间**: $(date '+%Y-%m-%d')
**最后更新**: $(date '+%Y-%m-%d %H:%M:%S')
**触发状态**: 自动执行
**集成状态**: 与Always规则集成
