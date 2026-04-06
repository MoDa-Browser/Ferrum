---
alwaysApply: false
description: 涉及代码修改后
---
1. 为核心功能编写单元测试，确保测试覆盖率 > 80%
2. 使用 Rust 内置 test 和 C++ Google Test 框架
3. 测试命名格式：`test_module_functionality`
4. 每个测试独立运行，避免依赖
5. 测试核心业务逻辑、数据处理、异常处理和边界情况
6. 定期运行静态代码分析工具
7. 进行安全测试和性能测试