#[cfg(test)]
pub fn env() -> std::collections::HashMap<String, String> {
    let env_str = std::fs::File::open(".env").unwrap();
    let env: std::collections::HashMap<String, String> = serde_json::from_reader(env_str).unwrap();
    env
}
