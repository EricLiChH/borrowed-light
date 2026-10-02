# E0502：借用发生冲突

| 元数据 | 内容 |
|---|---|
| 任务 | 消除重叠的只读借用与可变借用 |
| 概念 | immutable borrow、mutable borrow、借用活跃区间（NLL） |
| 错误码 | `E0502` |
| 先修 | `&T` 与 `&mut T` 的语义 |
| 项目阶段 | 领域模型、批量结果处理 |

## 现象

```rust,compile_fail
fn main() {
    let mut urls = vec![String::from("https://example.com")];
    let first = &urls[0];
    urls.push(String::from("https://rust-lang.org"));
    println!("{first}");
}
```

```text
error[E0502]: cannot borrow `urls` as mutable because it is also borrowed as immutable
   |
 3 |     let first = &urls[0];
   |                  ---- immutable borrow occurs here
 4 |     urls.push(String::from("https://rust-lang.org"));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
 5 |     println!("{first}");
   |                ----- immutable borrow later used here
```

## 判断

关键在最后一行：**冲突不是因为 `&mut` 本身，而是因为 `first` 在那之后还要被使用。**

**删掉哪一行就能编译？** 删掉 `println!("{first}")`。这说明判据是**借用的活跃区间**（从创建到最后一次使用），不是花括号范围。

## 修复选择

| 修法 | 什么时候用 | 代价 |
|---|---|---|
| 缩短借用范围 | 借用只在前面用到 | 无 |
| 调整操作顺序 | 可以先改后读 | 无 |
| 先克隆快照 | 循环中确实需要稳定的旧视图 | 一次分配 |
| 换 API（`retain`/`iter_mut`） | 想表达「原地修改」 | 无，通常更清晰 |
| 按索引访问 | 需要在下标计算与修改间穿插 | 越界会 panic，要小心 |

```rust,ignore
// 缩短范围：先算完再改
let first_len = urls[0].len();      // 借用在这里结束
urls.push(String::from("https://rust-lang.org"));

// 先收集再批量修改
let additions: Vec<String> = urls.iter().map(|url| format!("{url}/health")).collect();
urls.extend(additions);
```

## 遍历中修改：最常见的真实形态

```rust,compile_fail
fn main() {
    let mut statuses = vec![200_u16, 404, 500];
    for status in &statuses {
        if *status == 500 {
            statuses.push(503);
        }
    }
    println!("{statuses:?}");
}
```

编译器连怎么改都说了：`help: consider using an index-based loop instead, or collecting modifications into a separate collection`。按推荐顺序：**先收集再 extend** > **用 `retain`/`iter_mut`** > **按索引遍历**。

深入阅读：[所有权：移动、借用与可变借用](../guided/ownership.md#6-可变借用是独占权限)。
