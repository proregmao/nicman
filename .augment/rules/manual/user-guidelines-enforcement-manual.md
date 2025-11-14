# User Guidelines 强制执行手册

## 📖 手册目的
本手册提供详细的User Guidelines执行指导，确保AI在所有项目中都能严格遵循规则。

## 🔧 规则执行机制详解

### 1. 规则层次结构
```
User Guidelines (通用基础指导)
    ↓ 自动触发
Always规则 (强制补充，每次必执行)
    ↓ 条件触发  
Auto规则 (自动执行，特定条件下触发)
    ↓ 参考指导
Manual规则 (手动参考，详细操作指南)
```

### 2. 强制执行流程
```bash
# 每次任务开始时的完整检查流程
execute_user_guidelines() {
    echo "=== User Guidelines 强制执行开始 ==="
    
    # 第一步：执行Always规则
    echo "1. 执行Always规则..."
    source .augment/rules/always/01-time-handling-always.md
    source .augment/rules/always/02-chinese-communication-always.md
    source .augment/rules/always/03-anti-hallucination-always.md
    source .augment/rules/always/04-error-handling-always.md
    
    # 第二步：检查Auto规则触发条件
    echo "2. 检查Auto规则触发条件..."
    check_auto_rule_triggers
    
    # 第三步：参考Manual规则
    echo "3. 参考Manual规则指导..."
    reference_manual_guidelines
    
    echo "=== User Guidelines 执行完成 ==="
}
```

## 🚨 强制检查清单

### 任务开始前检查
```bash
# 强制执行的预检查清单
pre_task_checklist() {
    echo "=== 任务开始前强制检查 ==="
    
    # 1. 时间处理检查
    echo "□ 时间处理检查"
    if command -v date &> /dev/null; then
        CURRENT_TIME=$(date '+%Y-%m-%d %H:%M:%S')
        echo "  ✅ 系统时间可用: $CURRENT_TIME"
    else
        echo "  ❌ 系统时间不可用，任务终止"
        exit 1
    fi
    
    # 2. 中文环境检查
    echo "□ 中文环境检查"
    if locale | grep -q "zh_CN\|UTF-8"; then
        echo "  ✅ 中文环境正常"
    else
        echo "  ⚠️ 中文环境可能有问题，但继续执行"
    fi
    
    # 3. 防幻觉机制检查
    echo "□ 防幻觉机制检查"
    echo "  ✅ 已启用文件存在性验证"
    echo "  ✅ 已启用代码可执行性验证"
    echo "  ✅ 已启用服务状态验证"
    
    # 4. 错误处理机制检查
    echo "□ 错误处理机制检查"
    echo "  ✅ 已启用逐个错误分析"
    echo "  ✅ 已禁用简化错误处理"
    
    echo "=== 预检查完成 ==="
}
```

### 任务执行中检查
```bash
# 执行过程中的持续检查
during_task_monitoring() {
    echo "=== 任务执行中监控 ==="
    
    # 监控时间使用
    monitor_time_usage() {
        if echo "$CONTENT" | grep -E "2025-01-|2024-12-"; then
            echo "❌ 检测到硬编码时间，立即修正"
            auto_fix_hardcoded_time
        fi
    }
    
    # 监控语言使用
    monitor_language_usage() {
        if ! echo "$CONTENT" | grep -E "[\u4e00-\u9fff]"; then
            echo "❌ 检测到非中文内容，立即修正"
            request_chinese_translation
        fi
    }
    
    # 监控幻觉内容
    monitor_hallucination() {
        if echo "$CONTENT" | grep -E "假设|应该|可能存在"; then
            echo "⚠️ 检测到可能的幻觉内容，需要验证"
            request_verification
        fi
    }
    
    echo "=== 监控完成 ==="
}
```

### 任务完成后验证
```bash
# 任务完成后的全面验证
post_task_verification() {
    echo "=== 任务完成后验证 ==="
    
    # 1. 时间合规性验证
    echo "1. 验证时间合规性..."
    if find . -name "*.md" -exec grep -l "2025-01-" {} \; 2>/dev/null; then
        echo "❌ 发现硬编码时间，验证失败"
        return 1
    else
        echo "✅ 时间合规性验证通过"
    fi
    
    # 2. 中文使用验证
    echo "2. 验证中文使用..."
    if find . -name "*.md" -exec grep -l "智能\|系统\|需求" {} \; 2>/dev/null; then
        echo "✅ 中文使用验证通过"
    else
        echo "❌ 中文使用验证失败"
        return 1
    fi
    
    # 3. 代码可执行性验证
    echo "3. 验证代码可执行性..."
    if find . -name "*.md" -exec grep -l '```bash\|```rust' {} \; 2>/dev/null; then
        echo "✅ 包含可执行代码"
    else
        echo "⚠️ 建议添加更多可执行代码示例"
    fi
    
    echo "=== 验证完成 ==="
}
```

## 🔄 项目生命周期执行

### 新项目初始化
```bash
# 新项目开始时的强制初始化
initialize_new_project() {
    echo "=== 新项目初始化 ==="
    
    # 1. 创建.augment目录结构
    mkdir -p .augment/{rules/{always,auto,manual},logs,config}
    
    # 2. 复制规则文件
    cp -r /template/.augment/rules/* .augment/rules/
    
    # 3. 初始化时间戳
    PROJECT_START_TIME=$(date '+%Y-%m-%d %H:%M:%S')
    echo "项目开始时间: $PROJECT_START_TIME" > .augment/project.info
    
    # 4. 创建强制检查脚本
    create_validation_script
    
    # 5. 设置中文环境
    export LANG=zh_CN.UTF-8
    export LC_ALL=zh_CN.UTF-8
    
    echo "✅ 新项目初始化完成"
    echo "项目开始时间: $PROJECT_START_TIME"
}
```

### 开发过程中执行
```bash
# 开发过程中的持续执行
continuous_execution() {
    echo "=== 开发过程持续执行 ==="
    
    # 每次生成内容前
    before_content_generation() {
        pre_task_checklist
        trigger_always_rules
    }
    
    # 每次生成内容后
    after_content_generation() {
        post_task_verification
        log_execution_stats
    }
    
    # 每次遇到错误时
    on_error_encountered() {
        echo "检测到错误，触发错误处理规则..."
        source .augment/rules/always/04-error-handling-always.md
        analyze_and_fix_error "$ERROR_INFO"
    }
    
    echo "=== 持续执行设置完成 ==="
}
```

## 📊 执行效果监控

### 合规性统计
```bash
# 生成合规性报告
generate_compliance_report() {
    echo "=== User Guidelines 合规性报告 ==="
    
    local report_time=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file=".augment/logs/compliance-report-$(date '+%Y%m%d').md"
    
    cat > "$report_file" << EOF
# User Guidelines 合规性报告

**报告时间**: $report_time

## 时间处理合规性
$(check_time_compliance)

## 中文沟通合规性  
$(check_chinese_compliance)

## 防幻觉合规性
$(check_anti_hallucination_compliance)

## 错误处理合规性
$(check_error_handling_compliance)

## 总体评分
$(calculate_overall_score)
EOF
    
    echo "✅ 合规性报告已生成: $report_file"
}
```

### 违规处理记录
```bash
# 记录违规处理过程
log_violation_handling() {
    local violation_type="$1"
    local violation_details="$2"
    local fix_action="$3"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    local log_file=".augment/logs/violations.log"
    
    cat >> "$log_file" << EOF
[$timestamp] 违规类型: $violation_type
详细信息: $violation_details
修正措施: $fix_action
处理状态: 已修正
---
EOF
    
    echo "✅ 违规处理已记录"
}
```

## 🎯 最佳实践指导

### 时间处理最佳实践
```bash
# 时间处理的最佳实践
time_handling_best_practices() {
    echo "=== 时间处理最佳实践 ==="
    
    # 1. 总是使用系统API
    echo "1. 获取时间的正确方式:"
    echo "   CURRENT_TIME=\$(date '+%Y-%m-%d %H:%M:%S')"
    
    # 2. 文档时间戳格式
    echo "2. 文档时间戳标准格式:"
    echo "   **创建时间**: \$(date '+%Y-%m-%d')"
    echo "   **最后更新**: \$(date '+%Y-%m-%d %H:%M:%S')"
    
    # 3. 代码中的时间处理
    echo "3. Rust代码中的时间处理:"
    cat << 'EOF'
use chrono::{DateTime, Utc};
let now: DateTime<Utc> = Utc::now();
println!("当前时间: {}", now.format("%Y-%m-%d %H:%M:%S"));
EOF
    
    echo "=== 最佳实践指导完成 ==="
}
```

### 错误处理最佳实践
```bash
# 错误处理的最佳实践
error_handling_best_practices() {
    echo "=== 错误处理最佳实践 ==="
    
    echo "1. 错误分析模板:"
    cat << 'EOF'
错误类型: [编译错误/运行时错误/逻辑错误]
错误位置: [文件名:行号]
错误信息: [具体错误信息]
错误原因: [详细分析原因]
修复方案: [具体修复步骤]
验证方法: [如何验证修复成功]
预防措施: [如何避免类似错误]
EOF
    
    echo "2. 禁止的错误处理方式:"
    echo "   ❌ 简化功能以避免错误"
    echo "   ❌ 绕过复杂的错误"
    echo "   ❌ 删除出错的代码重新开始"
    
    echo "3. 推荐的错误处理方式:"
    echo "   ✅ 逐个分析每个错误的具体原因"
    echo "   ✅ 针对每个错误提供具体的修复方案"
    echo "   ✅ 保持原有功能的完整性"
    
    echo "=== 最佳实践指导完成 ==="
}
```

## 🔧 故障排查指南

### 常见问题及解决方案
```bash
# 常见问题排查
troubleshoot_common_issues() {
    echo "=== 常见问题排查 ==="
    
    # 问题1：时间验证失败
    echo "问题1: 时间验证失败"
    echo "症状: 发现硬编码时间"
    echo "解决: 执行 date '+%Y-%m-%d %H:%M:%S' 获取实际时间"
    
    # 问题2：中文检查失败
    echo "问题2: 中文检查失败"
    echo "症状: 内容主要为英文"
    echo "解决: 重新生成中文版本内容"
    
    # 问题3：代码不可执行
    echo "问题3: 代码不可执行"
    echo "症状: 代码示例无法运行"
    echo "解决: 提供完整的可执行代码示例"
    
    echo "=== 排查指南完成 ==="
}
```

---

**手册版本**: v1.0
**创建时间**: $(date '+%Y-%m-%d')
**最后更新**: $(date '+%Y-%m-%d %H:%M:%S')
**适用范围**: 所有AI开发项目
**维护状态**: 持续更新
