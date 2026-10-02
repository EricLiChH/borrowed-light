# 目标 URL 交给 url crate 解析并规范化

`MonitorTarget::new` 最初自己用 `split_once("://")` 和 `split(['/', '?', '#'])` 拆 URL。手写解析在小样本上能通过测试，但有真实的洞：scheme 大小写敏感（`HTTPS://` 被拒）、端口不校验、路径里的空格不检查，IPv6、IDN 和百分号编码完全没有处理。

改用 `url::Url` 之后，校验、规范化和解析合并成一步，且只发生一次：scheme 与 host 小写化，空路径补成 `/`。因此 `https://www.rust-lang.org` 以 `https://www.rust-lang.org/` 存储、比较和打印。

规范化的副作用是可见的：数据库里存的是规范形式，HTTP 响应与 CLI 输出也是。这是有意的——如果保留原始输入，同一个人写的 `http://EXAMPLE.com` 和 `http://example.com/` 会被当成两个目标，历史记录也就无法合并。`TargetError` 仍保持三个可匹配的类别（空名字、非法 URL、非 HTTP scheme），所以调用方的分支逻辑没有变化。

自己写解析器是课程里值得做一次的练习，但不该留在产品代码里。
