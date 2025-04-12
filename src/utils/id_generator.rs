//! ID生成ユーティリティモジュール
//! 
//! このモジュールには、一意のIDを生成するためのユーティリティ関数や構造体が含まれています。

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// シンプルなID生成器
pub struct IdGenerator {
    /// 次に生成するID値
    next_id: AtomicU32,
}

impl IdGenerator {
    /// 新しいID生成器を作成
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
        }
    }
    
    /// 新しいIDを作成
    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
    
    /// ID生成器をリセット
    pub fn reset(&self) {
        self.next_id.store(1, Ordering::SeqCst);
    }
    
    /// 現在のIDカウンタを取得
    pub fn current_count(&self) -> u32 {
        self.next_id.load(Ordering::SeqCst)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// タイムスタンプベースのUUID生成器
pub struct UuidGenerator {
    /// ノードID（装置固有の識別子）
    node_id: u16,
    /// シーケンスカウンタ
    sequence: AtomicU32,
}

impl UuidGenerator {
    /// 新しいUUID生成器を作成
    pub fn new(node_id: u16) -> Self {
        Self {
            node_id,
            sequence: AtomicU32::new(0),
        }
    }
    
    /// 新しいUUIDを生成
    pub fn generate(&self) -> String {
        // タイムスタンプを取得（ミリ秒）
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // シーケンス番号を増加
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst) % 0xFFFF;
        
        // ランダム部分（実際はJavaScriptのMath.random()相当の処理が必要）
        let random = js_sys::Math::random() * 0xFFFF as f64;
        
        // UUIDを組み立て
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            timestamp & 0xFFFFFFFF,
            (timestamp >> 32) & 0xFFFF,
            ((timestamp >> 48) & 0x0FFF) | 0x4000,  // バージョン4
            ((random as u32) & 0x3FFF) | 0x8000,    // バリアント
            ((self.node_id as u64) << 48) | ((sequence as u64) << 32) | (random as u64 & 0xFFFFFFFF)
        )
    }
}

/// ゲーム内で一意のエンティティIDを生成
/// 
/// # 戻り値
/// 
/// * 一意のエンティティID
pub fn generate_entity_id() -> u32 {
    static ENTITY_ID_GENERATOR: AtomicU32 = AtomicU32::new(1);
    ENTITY_ID_GENERATOR.fetch_add(1, Ordering::SeqCst)
}

/// 短い一意のセッションIDを生成
/// 
/// # 戻り値
/// 
/// * セッションID
pub fn generate_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    let random = (js_sys::Math::random() * 0xFFFFFF as f64) as u32;
    
    format!("{:x}{:06x}", timestamp & 0xFFFFFFFF, random & 0xFFFFFF)
}

/// ランダムな文字列IDを生成
/// 
/// # 引数
/// 
/// * `length` - 生成するIDの長さ
/// 
/// # 戻り値
/// 
/// * ランダムな文字列ID
pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    
    let mut result = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = (js_sys::Math::random() * CHARSET.len() as f64) as usize;
        result.push(CHARSET[idx] as char);
    }
    
    result
} 