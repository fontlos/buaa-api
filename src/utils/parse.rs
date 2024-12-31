// 用于匹配一个字符串中的某个标签对之间的内容
pub(crate) fn get_value_by_lable<'a>(text: &'a str, right: &str, left: &str) -> Option<&'a str> {
    if let Some(start) = text.find(right) {
        // 计算开始位置
        let value_start = start + right.len();
        // 查找结束位置
        if let Some(end) = text[value_start..].find(left) {
            // 返回字符串切片
            Some(&text[value_start..value_start + end])
        } else {
            // 理论上不可能出错
            None
        }
    } else {
        None
    }
}

// 用于匹配一个字符串中的多组标签对之间的内容
pub(crate) fn get_values_by_lable<'a>(text: &'a str, right: &str, left: &str) -> Vec<&'a str> {
    let mut values = Vec::new();
    let mut start_index = 0;

    while let Some(start) = text[start_index..].find(right) {
        // 计算值的起始位置
        let value_start = start_index + start + right.len();
        // 查找结束位置
        if let Some(end) = text[value_start..].find(left) {
            // 提取值并添加到 Vec 中
            values.push(&text[value_start..value_start + end]);
            // 更新 start_index 为当前匹配结束的位置，继续查找下一对标签
            start_index = value_start + end + left.len();
        } else {
            // 如果没有找到结束标签，退出循环
            break;
        }
    }

    values
}
