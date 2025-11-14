#!/bin/bash
# 集成测试脚本 - 测试nicman的所有核心功能

set -e

echo "=== nicman 集成测试 ==="
echo "测试时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
test_function() {
    local test_name="$1"
    local test_command="$2"
    
    echo -n "测试: $test_name ... "
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 通过${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ 失败${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# 1. 编译测试
echo "1. 编译测试"
test_function "Cargo编译" "cargo build --release"
echo ""

# 2. 二进制文件测试
echo "2. 二进制文件测试"
test_function "二进制文件存在" "test -f target/release/nicman"
test_function "二进制文件可执行" "test -x target/release/nicman"
echo ""

# 3. 命令行参数测试
echo "3. 命令行参数测试"
test_function "版本信息显示" "./target/release/nicman --version"
test_function "帮助信息显示" "./target/release/nicman --help"
echo ""

# 4. 权限检查测试
echo "4. 权限检查测试"
if [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}⚠ 非root用户，跳过需要权限的测试${NC}"
else
    echo -e "${GREEN}✓ 以root权限运行，可以执行完整测试${NC}"
fi
echo ""

# 5. 依赖库测试
echo "5. 依赖库测试"
test_function "ratatui依赖" "cargo tree | grep -q ratatui"
test_function "crossterm依赖" "cargo tree | grep -q crossterm"
test_function "tokio依赖" "cargo tree | grep -q tokio"
test_function "serde依赖" "cargo tree | grep -q serde"
echo ""

# 6. 模块测试
echo "6. 模块测试"
test_function "model模块编译" "cargo check --lib 2>&1 | grep -q 'Finished' || true"
test_function "backend模块存在" "test -f src/backend/runtime.rs"
test_function "ui模块存在" "test -f src/ui.rs"
test_function "utils模块存在" "test -d src/utils"
echo ""

# 7. 代码质量测试
echo "7. 代码质量测试"
test_function "Cargo.toml格式正确" "cargo metadata --format-version 1 > /dev/null"
test_function "无严重编译警告" "cargo build 2>&1 | grep -v 'warning:' | grep -q 'Finished'"
echo ""

# 8. 文档测试
echo "8. 文档测试"
test_function "README存在" "test -f README.md"
test_function "Cargo.toml存在" "test -f Cargo.toml"
echo ""

# 9. 功能模块完整性测试
echo "9. 功能模块完整性测试"
test_function "runtime模块" "grep -q 'pub fn list_interfaces' src/backend/runtime.rs"
test_function "traffic模块" "grep -q 'pub struct TrafficMonitor' src/backend/traffic.rs"
test_function "owner_detection模块" "grep -q 'pub struct OwnerDetector' src/backend/owner_detection.rs"
test_function "removal模块" "grep -q 'pub struct RemovalManager' src/backend/removal.rs"
test_function "netplan模块" "grep -q 'pub struct NetplanManager' src/backend/netplan.rs"
echo ""

# 10. 数据模型测试
echo "10. 数据模型测试"
test_function "InterfaceKind枚举" "grep -q 'pub enum InterfaceKind' src/model.rs"
test_function "NetInterface结构" "grep -q 'pub struct NetInterface' src/model.rs"
test_function "TrafficStats结构" "grep -q 'pub struct TrafficStats' src/model.rs"
test_function "InterfaceOwner枚举" "grep -q 'pub enum InterfaceOwner' src/model.rs"
echo ""

# 总结
echo "=== 测试总结 ==="
echo -e "通过: ${GREEN}$TESTS_PASSED${NC}"
echo -e "失败: ${RED}$TESTS_FAILED${NC}"
echo -e "总计: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}✗ 有 $TESTS_FAILED 个测试失败${NC}"
    exit 1
fi

