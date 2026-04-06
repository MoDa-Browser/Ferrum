---
alwaysApply: false
description: 版本控制规则
---
1. 从 main 分支创建功能分支，命名格式：`feature/MXXX-Description`
2. 编写清晰的提交消息，格式：`type(scope): description`
3. 提交前进行自我代码审查
4. 每个 Pull Request 必须经过至少一次代码审查
5. 合并到 main 分支前确保所有测试通过
6. 遵循语义化版本规范
7. 定期拉取最新更改，避免冲突
8. 推送前确保本地分支与远程分支同步
9. 当本地分支落后于远程分支时，使用 `git pull --no-edit` 拉取更改，避免打开终端编辑器
10. 拉取后解决所有冲突再进行推送
11. 推送时指定具体分支，格式：`git push origin branch-name`
12. 确保推送前所有测试通过
13. 推送后检查 CI/CD 状态，确保构建成功
14. 避免强制推送，请保持在得到用户的同意后再进行推送