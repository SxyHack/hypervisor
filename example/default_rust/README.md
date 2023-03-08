### 安装环境
1. 安装 2019 (必须)
2. 安装 Clang 编译器，VS单个组建搜索clang
3. 安装 WDK (ver2004)[https://go.microsoft.com/fwlink/?linkid=2128854]
4. 安装 Spectre 漏洞库(MSVC v142 -VS2019 C++ x64/x86 Spectre 缓解库(最新) )

### 重建项目时, 需要删除Cargo.toml, Cargo.lock

### 依赖库 & 参考
- https://crates.io/crates/modular_bitfield
- https://crates.io/crates/bitfield-struct
- https://os.phil-opp.com/zh-CN/minimal-rust-kernel/
  - 最小内核

#### bitfields
https://www.codercto.com/a/101174.html


### 踩到的坑
#### 1. 不能使用 DEBUG (⭐⭐⭐⭐⭐)
在 Debug 模式下, 调用IntrinsicT.cpuid会导致kernel模块触发FAST_FAIL事件.
调试很久没有头绪, 最后换成Release就可以了