# lab1

## _trace_request == 0

一开始没搞清楚情况，先用task context获取了sp，然后在sp + id的位置上read_volatile()，然后整个程序崩溃了。

后来发现因为没有分地址空间，所以用`&var as *const u8`获取的裸指针就是绝对地址，所以直接`(_id as *const u8).read_volatile()`就可以了。

## _trace_request == 1

同上，直接对`_id`的地址进行操作，用`unsafe`包裹之后写裸指针：

```rust
unsafe {
    (_id as *mut u8).write_volatile(_data as u8)
};
```

## _trace_request == 2

在`Task Control Block`中添加了一个`TaskInfo`的结构体，里面包含了一个`syscall_counter`数组，偷懒直接用数组下标表示系统调用号了。

为`Task Control Block`实现两个方法，分别读和写`syscall_counter`，然后在`TaskManager`里面获取current，对对应的TCB调用这两个方法，然后包一下`TASK_MANAGER`的实现。

最后在`syscall`的入口对相应的系统调用号加一（这里没考虑边界），然后在trace中调用打印。