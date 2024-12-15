pub(crate) fn get_value_by_lable(text: &str, right: &str, left: &str) -> Option<String> {
    if let Some(start) = text.find(right) {
        // 计算开始位置
        let value_start = start + right.len();
        // 查找结束位置
        if let Some(end) = text[value_start..].find(left) {
            // 提取值
            Some(String::from(&text[value_start..value_start + end]))
        } else {
            // 理论上不可能出错
            None
        }
    } else {
        None
    }
}
