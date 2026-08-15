fn successful_count(statuses: &[u16]) -> usize {
    // TODO: 用迭代器统计 200..300 的状态码，不要修改输入。
    let _ = statuses;
    todo!("count successful statuses")
}

fn main() {}

#[test]
fn counts_successes_without_changing_the_input() {
    let statuses = [200, 204, 301, 404, 503];
    assert_eq!(successful_count(&statuses), 2);
    assert_eq!(statuses, [200, 204, 301, 404, 503]);
}
