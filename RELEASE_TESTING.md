# 🧪 发布自动化测试流程

## 📋 测试目标

在 `feat/release-automation-testing` 分支上测试完整的发布自动化流程，不会实际发布到 crates.io。

## 🔧 测试配置

### 1. 测试工作流程
- `.github/workflows/release-test.yml` - 测试专用的 GitHub Actions
- 触发条件：`test-v*` 标签
- 只创建测试版本，不发布到 crates.io

### 2. 测试配置文件
- `release-test.toml` - cargo-release 测试配置
- 设置 `publish = false` 避免意外发布
- 使用 `test-v*` 标签格式

### 3. 测试脚本
- `scripts/test-release.sh` - 完整的测试脚本
- 验证所有预发布检查
- 测试 cargo-release 自动化

## 🚀 运行测试

### 方法一：使用测试脚本（推荐）
```bash
# 确保在正确的分支
git checkout feat/release-automation-testing

# 运行完整测试
./scripts/test-release.sh
```

### 方法二：手动步骤
```bash
# 1. 运行预发布检查
cargo test --workspace --all-features
cargo check --workspace --all-features
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings

# 2. 测试构建
cargo build --release --all-features

# 3. 测试发布（干运行）
cd hsipc-macros && cargo publish --dry-run
cd ../hsipc && cargo publish --dry-run

# 4. 测试 cargo-release
cargo release --config release-test.toml patch --no-publish
```

## 🔍 验证测试结果

### 本地验证
- ✅ 所有测试通过
- ✅ 代码质量检查通过
- ✅ 构建成功
- ✅ 包可以发布（dry-run）

### GitHub Actions 验证
- ✅ 创建了测试 tag (`test-v*`)
- ✅ 触发了 GitHub Actions
- ✅ 创建了测试版本（draft + prerelease）
- ✅ 所有 CI 步骤通过

### 检查点
1. **GitHub Actions**: https://github.com/loyalpartner/hsipc/actions
2. **测试 Release**: https://github.com/loyalpartner/hsipc/releases
3. **测试 Tag**: `git tag --list test-v*`

## 📝 测试场景

### 场景1：补丁版本测试
```bash
# 测试 0.1.0 → 0.1.1
cargo release --config release-test.toml patch --no-publish
```

### 场景2：次要版本测试
```bash
# 测试 0.1.0 → 0.2.0
cargo release --config release-test.toml minor --no-publish
```

### 场景3：主要版本测试
```bash
# 测试 0.1.0 → 1.0.0
cargo release --config release-test.toml major --no-publish
```

## 🎯 成功标准

测试成功的标准：
1. 所有预发布检查通过
2. 代码构建成功
3. 包验证通过（dry-run）
4. 测试 tag 创建成功
5. GitHub Actions 运行成功
6. 测试 Release 创建成功

## 🔄 从测试到生产

测试成功后，可以：
1. 创建 PR 合并到 master
2. 设置 `CRATES_IO_TOKEN` 秘钥
3. 使用生产配置 `release.toml`
4. 创建正式的版本标签

## 🚨 注意事项

- 测试标签使用 `test-v*` 格式，不会触发生产发布
- 测试版本标记为 draft 和 prerelease
- 不会发布到 crates.io
- 测试完成后可以删除测试标签和版本