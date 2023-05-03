# 如何实现？

在`TaskControlBlock`中增加`start_time`和`syscall_times`字段，给`TaskManager`增加`get_current_task_control_block`方法，然后复制给`task_info`

# 遇到的相关问题

由于`TaskControlBlock`中增加的`syscall_times`过大导致了栈溢出`stack overflow`，目前在`entry.asm`文件中增加了栈的size通过了。后续还是要思考如何减少开销。
特别感谢两位大佬`scpointer，weny`

# 解决 `stack overflow`问题
使用全局变量来记录`syscall times`，使用`unsafe rust`对 `static mut`变量进行访问。
```rust
// os/src/task/mod.rs
static mut TASK_SYSCALL_TIMES: [[u32; MAX_SYSCALL_NUM]; MAX_APP_NUM] =
    [[0; MAX_SYSCALL_NUM]; MAX_APP_NUM];
```