### 踩到的坑
#### 1. 不能使用 DEBUG (⭐⭐⭐⭐⭐)
在 Debug 模式下, 调用IntrinsicT.cpuid会导致kernel模块触发FAST_FAIL事件.
调试很久没有头绪, 最后换成Release就可以了


### 重建项目时, 需要删除Cargo.toml, Cargo.lock

### 
- https://crates.io/crates/modular_bitfield
- https://crates.io/crates/bitfield-struct
- https://os.phil-opp.com/zh-CN/minimal-rust-kernel/
  - 最小内核

### bitfields
https://www.codercto.com/a/101174.html

### Driver
Windows 10, version 2004 
https://go.microsoft.com/fwlink/?linkid=2128854