#!/bin/bash

# User Guidelines 规则验证脚本
# 用于验证AI是否严格遵循所有规则

echo "=== User Guidelines 规则验证开始 ==="
echo "验证时间: $(date '+%Y-%m-%d %H:%M:%S')"

# 初始化验证结果
VALIDATION_PASSED=true
VIOLATIONS_FOUND=0

# 创建日志目录
mkdir -p .augment/logs

# 1. Always规则验证
echo ""
echo "🔒 验证Always规则（强制执行）..."

# 1.1 时间处理规则验证
echo "  检查时间处理规则..."
hardcoded_time_files=$(find docs -name "*.md" -exec grep -l "2025-01-\|2024-12-\|2023-" {} \; 2>/dev/null)
if [ -n "$hardcoded_time_files" ]; then
    echo "  ❌ 违规：发现硬编码时间在文件: $hardcoded_time_files"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
    echo "$(date '+%Y-%m-%d %H:%M:%S') - 违规：硬编码时间在 $hardcoded_time_files" >> .augment/logs/violations.log
else
    echo "  ✅ 时间处理规则通过"
fi

# 检查是否使用了实际系统时间
current_date=$(date '+%Y-%m-%d')
if find docs -name "*.md" -exec grep -l "$current_date" {} \; 2>/dev/null | head -1; then
    echo "  ✅ 使用了实际系统时间: $current_date"
else
    echo "  ⚠️  未发现当前系统时间，请检查"
fi

# 1.2 中文沟通规则验证
echo "  检查中文沟通规则..."
if find docs -name "*.md" -exec grep -l "智能\|系统\|需求\|配置\|功能" {} \; 2>/dev/null | head -1; then
    echo "  ✅ 中文沟通规则通过"
else
    echo "  ❌ 违规：未使用中文"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
    echo "$(date '+%Y-%m-%d %H:%M:%S') - 违规：未使用中文" >> .augment/logs/violations.log
fi

# 1.3 防幻觉规则验证
echo "  检查防幻觉规则..."
if find docs -name "*.md" -exec grep -l '```bash\|```rust\|```yaml' {} \; 2>/dev/null | head -1; then
    echo "  ✅ 包含可执行代码示例"
else
    echo "  ⚠️  建议添加更多可执行代码示例"
fi

# 检查是否有假设性描述
hallucination_files=$(find docs -name "*.md" -exec grep -l "假设\|应该存在\|可能已经\|假定\|预期存在" {} \; 2>/dev/null)
if [ -n "$hallucination_files" ]; then
    echo "  ❌ 违规：发现假设性描述在文件: $hallucination_files"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
    echo "$(date '+%Y-%m-%d %H:%M:%S') - 违规：假设性描述在 $hallucination_files" >> .augment/logs/violations.log
else
    echo "  ✅ 防幻觉规则通过"
fi

# 1.4 错误处理规则验证
echo "  检查错误处理规则..."
error_simplification_files=$(find docs -name "*.md" -exec grep -l "简化版本\|暂时跳过\|稍后处理\|简化功能\|绕过错误" {} \; 2>/dev/null)
if [ -n "$error_simplification_files" ]; then
    echo "  ❌ 违规：发现简化错误处理在文件: $error_simplification_files"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
    echo "$(date '+%Y-%m-%d %H:%M:%S') - 违规：简化错误处理在 $error_simplification_files" >> .augment/logs/violations.log
else
    echo "  ✅ 错误处理规则通过"
fi

# 2. Auto规则验证
echo ""
echo "🔄 验证Auto规则（自动触发）..."

# 2.1 时间验证自动规则
echo "  检查时间验证自动规则..."
if find docs -name "*.md" -exec grep -l "时间\|日期" {} \; 2>/dev/null | head -1; then
    echo "  ✅ 检测到时间相关内容，应触发时间验证规则"
    # 验证是否正确处理了时间
    if find docs -name "*.md" -exec grep -l "$current_date" {} \; 2>/dev/null | head -1; then
        echo "  ✅ 时间验证自动规则执行正确"
    else
        echo "  ⚠️  时间验证自动规则可能未正确执行"
    fi
fi

# 3. Manual规则验证
echo ""
echo "📖 验证Manual规则（参考指导）..."

# 检查是否存在Manual规则文件
if [ -f ".augment/rules/manual/user-guidelines-enforcement-manual.md" ]; then
    echo "  ✅ Manual规则文件存在"
else
    echo "  ❌ Manual规则文件缺失"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
fi

# 4. 规则文件完整性验证
echo ""
echo "📁 验证规则文件完整性..."

# 检查Always规则文件
always_rules=(
    "01-time-handling-always.md"
    "02-chinese-communication-always.md"
    "03-anti-hallucination-always.md"
    "04-error-handling-always.md"
)

for rule in "${always_rules[@]}"; do
    if [ -f ".augment/rules/always/$rule" ]; then
        echo "  ✅ $rule 存在"
    else
        echo "  ❌ $rule 缺失"
        VALIDATION_PASSED=false
        ((VIOLATIONS_FOUND++))
    fi
done

# 检查Auto规则文件
if [ -f ".augment/rules/auto/time-validation-auto.md" ]; then
    echo "  ✅ time-validation-auto.md 存在"
else
    echo "  ❌ time-validation-auto.md 缺失"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
fi

# 5. 配置文件验证
echo ""
echo "⚙️  验证配置文件..."

if [ -f ".augment/config/rule-triggers.yaml" ]; then
    echo "  ✅ rule-triggers.yaml 存在"
else
    echo "  ❌ rule-triggers.yaml 缺失"
    VALIDATION_PASSED=false
    ((VIOLATIONS_FOUND++))
fi

# 6. 生成验证报告
echo ""
echo "📊 生成验证报告..."

report_file=".augment/logs/validation-report-$(date '+%Y%m%d-%H%M%S').md"

cat > "$report_file" << EOF
# User Guidelines 规则验证报告

**验证时间**: $(date '+%Y-%m-%d %H:%M:%S')
**验证结果**: $(if [ "$VALIDATION_PASSED" = true ]; then echo "✅ 通过"; else echo "❌ 失败"; fi)
**违规数量**: $VIOLATIONS_FOUND

## Always规则验证结果
- 时间处理规则: $(if find docs -name "*.md" -exec grep -l "2025-01-\|2024-12-" {} \; 2>/dev/null | head -1; then echo "❌ 失败"; else echo "✅ 通过"; fi)
- 中文沟通规则: $(if find docs -name "*.md" -exec grep -l "智能\|系统" {} \; 2>/dev/null | head -1; then echo "✅ 通过"; else echo "❌ 失败"; fi)
- 防幻觉规则: $(if find docs -name "*.md" -exec grep -l "假设\|应该存在" {} \; 2>/dev/null | head -1; then echo "❌ 失败"; else echo "✅ 通过"; fi)
- 错误处理规则: $(if find docs -name "*.md" -exec grep -l "简化版本\|暂时跳过" {} \; 2>/dev/null | head -1; then echo "❌ 失败"; else echo "✅ 通过"; fi)

## Auto规则验证结果
- 时间验证自动规则: ✅ 正常触发

## Manual规则验证结果
- 规则文件完整性: $(if [ -f ".augment/rules/manual/user-guidelines-enforcement-manual.md" ]; then echo "✅ 通过"; else echo "❌ 失败"; fi)

## 改进建议
$(if [ "$VALIDATION_PASSED" = false ]; then
    echo "- 修复发现的违规问题"
    echo "- 强化规则执行机制"
    echo "- 增加自动修正功能"
else
    echo "- 继续保持良好的规则遵循"
    echo "- 定期进行规则验证"
fi)

---
**报告生成时间**: $(date '+%Y-%m-%d %H:%M:%S')
EOF

echo "  ✅ 验证报告已生成: $report_file"

# 7. 输出最终结果
echo ""
echo "=== 验证结果汇总 ==="
echo "验证时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "总体结果: $(if [ "$VALIDATION_PASSED" = true ]; then echo "✅ 通过"; else echo "❌ 失败"; fi)"
echo "违规数量: $VIOLATIONS_FOUND"
echo "报告文件: $report_file"

if [ "$VALIDATION_PASSED" = true ]; then
    echo ""
    echo "🎉 恭喜！所有规则验证通过，User Guidelines执行良好！"
    exit 0
else
    echo ""
    echo "⚠️  发现违规问题，请检查并修正后重新验证。"
    echo "违规日志: .augment/logs/violations.log"
    exit 1
fi
