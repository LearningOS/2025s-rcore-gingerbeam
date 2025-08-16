# lab2

## reimplement `sys_get_time`

创建新的 `TimeVal` 对象的方法和 ch3 的处理相同，然后仍然需要写入。

需要写入的 `TimeVal` 对象由用户传入，在用户空间中，系统调用需要在S Mode访问用户空间的地址，需要用 `current_user_token` 获取当前用户空间的页表基地址，然后translate。

同时可能存在这个对象不在一个页面上的情况，最好的办法是一个字节一个字写入，好在 `page_table` 提供了 translate 并获取指定长度的字节序列的方法。

## reimplement `sys_trace`

其实就是在内存管理的场景下如何实现读取/写入用户空间中一个地址的问题。

其实上一个调用应该也能这样实现，但是这边直接包到PTE一层了，上面还是需要调用page table，相当于少写了两个函数。

下面的逻辑其实不难，即通过va获取vpn和offset，通过translate获取vpn对应的pte，通过pte获取权限和ppn，通过ppn实现的get_bytes_array方法获取改物理页帧上面的字节，然后根据offset（作为下标）获取对应的地址上的数据。写数据也是同理，只是将读字节换成了写。

## implement `sys_mmap`

先检查地址对齐、权限是否合法，然后构造一个VPNRange对象（使用start和len），遍历VPNRange中的所有vpn，检查是否已经由映射。所有测试通过进入下一步。

先获取权限，在port后面加上PTE_U权限，然后在当前task control block的memory set中的采用`insert_framed_area`方法，创建一个framed类型的MapArea，MapArea在new的时候会调用frame allocator申请物理页帧，并且维护页表的映射。

## implement `sys_munmap`

反过来的逻辑也很简单，检查合法问题之后，在当前task control block的memory set中调用`unmap`方法（自行实现），通过memory set调用private的page table来unmap。此操作对每一个合法的VPN都做一次。
