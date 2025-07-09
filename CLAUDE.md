## 开发规范

- 文档放在 docs 目录
- 查看 @README.md 了解项目概述
- git 工作流程 @docs/git-instructions.md, 按照这个流程开发
- 测试流程 @docs/TESTING.md
- 架构文档 @docs/ARCHITECTURE.md
- API 设计 @docs/API.md
- 性能测试 @docs/PERFORMANCE.md

当我们开始开发一个新功能时，请遵循以下规范：


- 先更新相关的文档，我确认过后再开始执行
- 我们采用的是 tdd 的开发模式
- Readme.md 使用中文
- 注释使用英文
- 不允许使用 println 宏，库的日志请使用 log 日志库，日志级别请使用 info 级别, 用 trace 级别
- 错误需要用 thiserror 重构下
