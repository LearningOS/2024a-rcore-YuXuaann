# 实现功能

1. 实现了银行家算法的安全性检测（死锁检测），但是因为网站上讲的银行家算法不太清楚，导致写了一版，结果寄一个点死活调不出来，于是问了我的队友(github id: crpboy)，然后懂了他的思路写了完全新的一版，不仅更好维护了，逻辑也更加清楚了

# 简答题

1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源 需要回收的资源有哪些？其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？  
    - 需要回收的资源有：`TaskControlBlock`
    - 其他线程的 `TaskControlBlock` 可能在：`waittid` 系统调用、切换线程的函数、在这两者中都会回收；关于信号量和锁的系统调用函数中（获取线程号），不会回收

2. 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？

```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```
第一种实现有可能让多个线程拿到锁
第二种只有当如果有等待的线程，才把锁的所有权移交给该线程

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

[crpboy](https://github.com/crpboy)

1. 此外，我也参考了**以下资料**，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Camp-Guide-2024A](https://learningos.cn/rCore-Camp-Guide-2024A)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
