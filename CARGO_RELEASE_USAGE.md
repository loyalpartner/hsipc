# 🚀 使用 cargo-release 进行完全自动化发布

## 📋 配置完成

已经完成了以下配置：
- ✅ 安装了 `cargo-release` 工具
- ✅ 创建了 `release.toml` 配置文件
- ✅ 添加了 LICENSE 文件
- ✅ 更新了 Cargo.toml 的发布信息

## 🎯 使用方法

### 1. 日常版本发布

```bash
# 发布补丁版本 (0.1.0 → 0.1.1)
cargo release patch

# 发布次要版本 (0.1.0 → 0.2.0)
cargo release minor

# 发布主要版本 (0.1.0 → 1.0.0)
cargo release major
```

### 2. 预览模式（推荐先使用）

```bash
# 查看将要执行的操作，不实际执行
cargo release patch --dry-run

# 查看详细信息
cargo release patch --dry-run --verbose
```

### 3. 自定义版本

```bash
# 发布特定版本
cargo release 0.1.5

# 发布预发布版本
cargo release 0.2.0-alpha.1
```

## 🔧 cargo-release 会自动做什么

1. **检查和测试**
   - 运行 `cargo test --all-features`
   - 运行 `cargo check --all-features`
   - 运行 `cargo clippy --all-features -- -D warnings`

2. **更新版本**
   - 自动更新 `Cargo.toml` 中的版本号
   - 更新 `CHANGELOG.md` 文件

3. **Git 操作**
   - 创建版本提交
   - 创建 git tag (如 `v0.1.1`)
   - 推送到 GitHub

4. **发布到 crates.io**
   - 先发布 `hsipc-macros`
   - 等待 300 秒确保可用
   - 再发布 `hsipc`

5. **触发 GitHub Actions**
   - 自动创建 GitHub Release
   - 生成文档并部署

## ⚠️ 发布前检查

在发布前，请确保：

1. **代码质量**
   ```bash
   # 运行完整测试
   cargo test --all-features
   
   # 检查代码格式
   cargo fmt --check
   
   # 运行 clippy
   cargo clippy --all-features -- -D warnings
   ```

2. **文档更新**
   - 更新 `CHANGELOG.md` 的 `[Unreleased]` 部分
   - 确保 `README.md` 信息是最新的

3. **设置 crates.io token**
   ```bash
   # 需要设置 GitHub Secret: CRATES_IO_TOKEN
   # 或者在本地设置环境变量
   export CARGO_REGISTRY_TOKEN=your_token_here
   ```

## 🎯 完整发布流程示例

```bash
# 1. 确保在 master 分支
git checkout master
git pull origin master

# 2. 预览发布
cargo release patch --dry-run

# 3. 实际发布
cargo release patch

# 4. 验证发布
# 检查 GitHub: https://github.com/loyalpartner/hsipc/releases
# 检查 crates.io: https://crates.io/crates/hsipc
```

## 🛠️ 故障排除

### 如果发布失败：

1. **检查 crates.io token**
   ```bash
   cargo login
   ```

2. **检查网络连接**
   ```bash
   cargo publish --dry-run
   ```

3. **检查依赖关系**
   ```bash
   cargo check --all-features
   ```

4. **回滚版本**
   ```bash
   # 如果需要回滚 git tag
   git tag -d v0.1.1
   git push origin :refs/tags/v0.1.1
   ```

## 🔄 与 GitHub Actions 集成

发布 tag 后，GitHub Actions 会自动：
- 创建 GitHub Release
- 生成变更日志
- 部署文档到 GitHub Pages
- 运行完整的测试套件

整个流程完全自动化，你只需要运行 `cargo release` 命令！