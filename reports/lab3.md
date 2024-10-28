# 实现功能

1. 使用了 `BinaryHeap` 进行调度

2. 实现 `sys_spawn` 系统调用

# 简答题

1. 实际情况是轮到 `p1` 执行吗？为什么？
不是，因为 `p1` 的 `pass` 溢出了

2. 为什么？尝试简单说明（不要求严格证明）。

    因为
    $$
    Pass = \frac{BigStride}{Priority}
    $$
    那么如果 $Priority \ge 2$
    $$
    Pass \le \frac{BigStride}{2}
    $$

    利用归纳法：
    - 初始时刻，满足
    $$
    Stride_{max} – Stride_{min} \le \frac{BigStride}{2}
    $$
    - 某一时刻，假设满足
    $$
    Stride_{max} – Stride_{min} \le \frac{BigStride}{2}
    $$
    那么下一时刻，调度 `stride` 最小的进程，使其前进 `pass` 长度，它要么成为 `stride` 最大的进程，要么成为 `stride` 非最大的进程。  
    - 若成为了 `stride` 最大的进程，设新的 `stride` 最小的进程为 $Stride_{min}'$，新的 `stride` 最大的进程为 $Stride_{min} + Pass$，那么
    $$
    Stride_{min} + Pass - Stride_{min}' \le Stride_{min} + Pass - Stride_{min} = Pass \le \frac{BigStride}{2}
    $$
    - 若成为了 `stride` 非最大的进程，设新的 `stride` 最小的进程为 $Stride_{min}'$，新的 `stride` 最大的进程仍为 $Stride_{max}$，那么
    $$
    Stride_{max} - Stride_{min}' \le Stride_{max} - Stride_{min} \le \frac{BigStride}{2}
    $$
    综上，对于任意时刻，都满足
    $$
    Stride_{max} – Stride_{min} \le \frac{BigStride}{2}
    $$

3. 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

    ```rust
    use core::cmp::Ordering;

    struct Stride(u64);

    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            if self.0 < other.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
    }

    impl PartialEq for Stride {
        fn eq(&self, other: &Self) -> bool {
            false
        }
    }
    ```

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了**以下资料**，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Camp-Guide-2024A](https://learningos.cn/rCore-Camp-Guide-2024A)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
