# 如何实现？

在`TaskControlBlock`中增加`start_time`和`syscall_times`字段，给`TaskManager`增加`get_current_task_control_block`方法，然后复制给`task_info`
