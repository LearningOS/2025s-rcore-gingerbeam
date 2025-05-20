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

## 问答题

1. Exception::IllegalInstruction RustSBI 0.3.0-alpha.2

2. trap.S

1. 在ch3中，sp此时已经指向了需要恢复的TrapContext（内核栈sp）；`__restore`的功能是加载一个TrapContext然后通过`sret`恢复到U mode执行，这个TrapContext可以是之前保存的，然后从trap返回继续执行，也可能是设置好的，用于从S mode进入U mode开始执行一个新的程序。

2. 先从内核栈中保存的信息将sstatus, sepc和sp三个寄存器恢复到通用寄存器中，然后写入硬件。sstatus表示进入trap之前的特权级，sepc表示进入trap之前的pc，sp则表示用户栈栈顶（存入sscratch）

3. 因为sp上面处理过了，tp没有用

4. sp此时指向用户栈栈顶，而sscratch保存内核栈栈顶

5. sret，sret会修改sstatus中的spp（用户栈和内核栈与cpu无关）

6. 进入trap处理流程之后，需要将sp指向内核栈栈顶，用sscratch保存用户栈栈顶

7. `ecall` 发起系统调用，cpu自动执行spp的修改和一系列硬件机制

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

deepseek-V3-0324在智能指针和裸指针相关语法上为我提供了许多帮助

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

deepseek-V3-0324在智能指针和裸指针相关语法上为我提供了许多帮助

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。