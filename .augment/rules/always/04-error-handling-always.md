# 错误处理强制规则

## 🚨 强制执行声明
**AI必须逐个分析和修复每个错误，绝对禁止简化、绕过或忽略任何错误。**

## 📋 规则内容

### 🚫 绝对禁止的错误处理方式
- ❌ 简化功能以避免错误
- ❌ 绕过复杂的错误
- ❌ 删除出错的代码重新开始
- ❌ 忽略编译或运行时错误
- ❌ 使用"稍后修复"的借口
- ❌ 提供不完整的解决方案

### ✅ 强制要求的错误处理方式
- ✅ 逐个分析每个错误的具体原因
- ✅ 针对每个错误提供具体的修复方案
- ✅ 保持原有功能的完整性
- ✅ 提供详细的错误解释
- ✅ 验证修复后的结果
- ✅ 提供预防类似错误的建议

## 🔧 强制执行机制

### 错误分析模板
```bash
# 每次遇到错误时必须按此模板分析
echo "=== 错误分析报告 ==="
echo "错误类型: [编译错误/运行时错误/逻辑错误]"
echo "错误位置: [文件名:行号]"
echo "错误信息: [具体错误信息]"
echo "错误原因: [详细分析原因]"
echo "修复方案: [具体修复步骤]"
echo "验证方法: [如何验证修复成功]"
echo "预防措施: [如何避免类似错误]"
echo "========================"
```

### 编译错误处理流程
```rust
// 示例：处理Rust编译错误
// ❌ 错误代码
fn main() {
    let x = 5;
    let y = "hello";
    let z = x + y;  // 编译错误：类型不匹配
    println!("{}", z);
}

// ✅ 错误分析和修复
/*
错误分析：
- 错误类型: 编译错误
- 错误位置: main.rs:4
- 错误信息: cannot add `&str` to `{integer}`
- 错误原因: 尝试将整数和字符串相加，类型不兼容
- 修复方案: 将整数转换为字符串或使用格式化宏
*/

fn main() {
    let x = 5;
    let y = "hello";
    // 修复方案1：格式化输出
    let z = format!("{}{}", x, y);
    println!("{}", z);
    
    // 修复方案2：字符串连接
    let z2 = x.to_string() + y;
    println!("{}", z2);
}
```

### 运行时错误处理流程
```rust
// 示例：处理运行时错误
use std::fs;

// ❌ 可能出错的代码
fn read_config() -> String {
    fs::read_to_string("config.yaml").unwrap()  // 可能panic
}

// ✅ 错误分析和修复
/*
错误分析：
- 错误类型: 运行时错误
- 错误位置: 文件读取操作
- 错误信息: No such file or directory
- 错误原因: 配置文件不存在
- 修复方案: 添加错误处理和默认配置
*/

fn read_config() -> Result<String, Box<dyn std::error::Error>> {
    match fs::read_to_string("config.yaml") {
        Ok(content) => {
            println!("✅ 成功读取配置文件");
            Ok(content)
        },
        Err(e) => {
            println!("⚠️ 配置文件读取失败: {}", e);
            println!("正在创建默认配置文件...");
            
            let default_config = r#"
# 默认配置文件
version: "1.0"
server:
  host: "localhost"
  port: 8080
"#;
            
            fs::write("config.yaml", default_config)?;
            println!("✅ 已创建默认配置文件");
            Ok(default_config.to_string())
        }
    }
}
```

## 🔍 验证机制

### 错误修复验证脚本
```bash
#!/bin/bash
# 错误修复验证脚本

echo "=== 错误修复验证 ==="

# 1. 编译验证
echo "1. 编译验证..."
if cargo check; then
    echo "✅ 编译通过"
else
    echo "❌ 编译失败，需要继续修复"
    cargo check 2>&1 | head -20
    exit 1
fi

# 2. 测试验证
echo "2. 测试验证..."
if cargo test; then
    echo "✅ 测试通过"
else
    echo "❌ 测试失败，需要修复测试"
    cargo test 2>&1 | head -20
    exit 1
fi

# 3. 功能验证
echo "3. 功能验证..."
if cargo run --example basic_test; then
    echo "✅ 基本功能正常"
else
    echo "❌ 功能异常，需要修复"
    exit 1
fi

echo "=== 验证完成 ==="
```

### 错误处理质量检查
```bash
#!/bin/bash
# 错误处理质量检查

echo "=== 错误处理质量检查 ==="

# 检查是否有unwrap()调用（可能导致panic）
echo "1. 检查潜在panic点..."
if grep -r "\.unwrap()" src/; then
    echo "⚠️ 发现unwrap()调用，建议使用错误处理"
else
    echo "✅ 没有发现unwrap()调用"
fi

# 检查是否有TODO或FIXME注释
echo "2. 检查未完成项..."
if grep -r "TODO\|FIXME" src/; then
    echo "⚠️ 发现未完成项，需要处理"
else
    echo "✅ 没有发现未完成项"
fi

# 检查错误处理覆盖率
echo "3. 检查错误处理覆盖率..."
error_handling_count=$(grep -r "Result\|Error\|match.*Err" src/ | wc -l)
function_count=$(grep -r "fn " src/ | wc -l)
if [ $error_handling_count -gt $((function_count / 2)) ]; then
    echo "✅ 错误处理覆盖率良好"
else
    echo "⚠️ 错误处理覆盖率偏低，建议增加"
fi

echo "=== 检查完成 ==="
```

## 🚨 违规处理

### 发现简化错误处理时的处理流程
1. **立即停止** - 停止简化或绕过错误
2. **详细分析** - 对每个错误进行详细分析
3. **逐个修复** - 针对每个错误提供具体修复方案
4. **功能验证** - 确保修复后功能完整
5. **测试验证** - 运行测试确保修复成功

### 常见违规修正示例

#### 简化功能 → 完整修复
```rust
// ❌ 违规示例：简化功能
fn simple_version() {
    println!("简化版本，暂时不处理错误");
}

// ✅ 修正：完整错误处理
fn complete_version() -> Result<(), Box<dyn std::error::Error>> {
    // 完整的功能实现，包含所有错误处理
    match std::fs::read_to_string("config.yaml") {
        Ok(content) => {
            println!("配置内容: {}", content);
            // 处理配置内容的逻辑
            Ok(())
        },
        Err(e) => {
            eprintln!("读取配置失败: {}", e);
            // 创建默认配置的逻辑
            std::fs::write("config.yaml", "default: true")?;
            println!("已创建默认配置");
            Ok(())
        }
    }
}
```

#### 绕过错误 → 直面错误
```rust
// ❌ 违规示例：绕过错误
fn bypass_error() {
    // 跳过复杂的网络请求，直接返回假数据
    println!("假设网络请求成功");
}

// ✅ 修正：处理真实错误
use reqwest;
use tokio;

#[tokio::main]
async fn handle_real_error() -> Result<(), Box<dyn std::error::Error>> {
    match reqwest::get("https://api.example.com/data").await {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    println!("请求成功: {}", text);
                    Ok(())
                },
                Err(e) => {
                    eprintln!("读取响应失败: {}", e);
                    Err(e.into())
                }
            }
        },
        Err(e) => {
            eprintln!("网络请求失败: {}", e);
            println!("尝试使用缓存数据...");
            // 实现缓存逻辑
            Ok(())
        }
    }
}
```

## 🔄 触发条件

### 自动触发情况
- 遇到任何编译错误
- 遇到任何运行时错误
- 遇到任何逻辑错误
- 发现任何异常情况
- 测试失败时

### 强制检查点
- 每次代码编译前
- 每次功能测试前
- 每次代码提交前
- 每次发现错误时

## 📊 执行统计

### 成功标准
- ✅ 所有错误都得到详细分析
- ✅ 所有错误都有具体修复方案
- ✅ 所有修复都经过验证
- ✅ 原有功能保持完整

### 质量指标
- 错误分析完整率 = 100%
- 错误修复成功率 = 100%
- 功能完整性保持率 = 100%
- 测试通过率 = 100%

---

**规则版本**: v1.0
**创建时间**: $(date '+%Y-%m-%d')
**最后更新**: $(date '+%Y-%m-%d %H:%M:%S')
**执行状态**: 强制执行
**违规容忍度**: 零容忍
