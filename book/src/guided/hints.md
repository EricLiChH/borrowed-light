# 分层提示

只展开当前需要的一层。每多看一层，自己探索的空间就少一点。

## 01_move

<details><summary>第一层：方向</summary>

编译器指出的是“移动之后又读取”。找到真正发生移动的那一行。

</details>

<details><summary>第二层：关键机制</summary>

`Vec::push` 的参数是值，不是引用；它会把元素移入 `Vec`。调试输出应该发生在移动之前，或改为读取 `Vec` 中的值。

</details>

<details><summary>第三层：改动位置</summary>

把 `println!` 移到 `queue.push(target)` 之前。不要为通过编译而 clone。

</details>

## 02_borrow

<details><summary>第一层：方向</summary>

`label` 只需要读取目标，没有理由接管它。

</details>

<details><summary>第二层：关键机制</summary>

把参数从 `MonitorTarget` 改成 `&MonitorTarget`；调用处也要借用。

</details>

<details><summary>第三层：改动位置</summary>

函数签名使用 `fn label(target: &MonitorTarget)`，调用使用 `label(&target)`。

</details>

## 03_mut_borrow

<details><summary>第一层：方向</summary>

函数需要修改 URL，因此普通引用不够；调用者也必须声明值可变。

</details>

<details><summary>第二层：关键机制</summary>

同时修改 `let target`、函数参数和调用表达式三处的可变性。

</details>

<details><summary>第三层：改动位置</summary>

使用 `let mut target`、`target: &mut MonitorTarget` 和 `normalize(&mut target)`。

</details>
