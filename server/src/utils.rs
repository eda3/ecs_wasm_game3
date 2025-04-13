use std::time::{SystemTime, UNIX_EPOCH};

/// 現在時刻のUNIXタイムスタンプを取得（秒）
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 現在時刻のUNIXタイムスタンプを取得（ミリ秒）
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// ログ出力用のエラー表示
pub fn log_error<T: std::fmt::Display>(err: &T) -> String {
    format!("Error: {}", err)
}

/// クエリパラメータをパース
pub fn parse_query_params(query: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    for pair in query.split('&') {
        let mut iter = pair.split('=');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            params.insert(key.to_string(), value.to_string());
        }
    }
    params
}

/// 文字列をパース
pub fn parse_str<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.parse::<T>().ok()
}

/// 共有しやすいURLセーフなIDを生成
pub fn generate_short_id() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const ID_LEN: usize = 6;
    
    let mut rng = rand::thread_rng();
    
    (0..ID_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// レート制限のためのシンプルな実装
pub struct RateLimiter {
    /// ウィンドウ時間（ミリ秒）
    window_ms: u64,
    
    /// ウィンドウ内の最大リクエスト数
    max_requests: usize,
    
    /// リクエスト履歴
    requests: Vec<u64>,
}

impl RateLimiter {
    /// 新しいレート制限器を作成
    pub fn new(window_ms: u64, max_requests: usize) -> Self {
        Self {
            window_ms,
            max_requests,
            requests: Vec::with_capacity(max_requests),
        }
    }
    
    /// リクエストを行えるかチェック
    pub fn check(&mut self) -> bool {
        let now = current_timestamp_ms();
        
        // 現在のウィンドウより古いリクエストを削除
        self.requests.retain(|&time| now - time < self.window_ms);
        
        // リクエスト数がリミット未満なら許可
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
    
    /// 次のリクエストが可能になるまでの待ち時間（ミリ秒）
    pub fn wait_time(&self) -> u64 {
        if self.requests.len() < self.max_requests {
            return 0;
        }
        
        let now = current_timestamp_ms();
        
        if let Some(&oldest) = self.requests.first() {
            let window_end = oldest + self.window_ms;
            if window_end > now {
                window_end - now
            } else {
                0
            }
        } else {
            0
        }
    }
} 