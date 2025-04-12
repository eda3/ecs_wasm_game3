//! キーコードの定数定義
//!
//! このモジュールはキーボードキーとマウスボタンの定数を定義します。
//! よく使われるキーとボタンには定数名が付いています。

/// マウス左ボタン
pub const MOUSE_LEFT: u8 = 0;
/// マウス右ボタン
pub const MOUSE_RIGHT: u8 = 1;
/// マウス中央ボタン
pub const MOUSE_MIDDLE: u8 = 2;

/// キーボード: A
pub const KEY_A: u32 = 65;
/// キーボード: B
pub const KEY_B: u32 = 66;
/// キーボード: C
pub const KEY_C: u32 = 67;
/// キーボード: D
pub const KEY_D: u32 = 68;
/// キーボード: E
pub const KEY_E: u32 = 69;
/// キーボード: F
pub const KEY_F: u32 = 70;
/// キーボード: G
pub const KEY_G: u32 = 71;
/// キーボード: H
pub const KEY_H: u32 = 72;
/// キーボード: I
pub const KEY_I: u32 = 73;
/// キーボード: J
pub const KEY_J: u32 = 74;
/// キーボード: K
pub const KEY_K: u32 = 75;
/// キーボード: L
pub const KEY_L: u32 = 76;
/// キーボード: M
pub const KEY_M: u32 = 77;
/// キーボード: N
pub const KEY_N: u32 = 78;
/// キーボード: O
pub const KEY_O: u32 = 79;
/// キーボード: P
pub const KEY_P: u32 = 80;
/// キーボード: Q
pub const KEY_Q: u32 = 81;
/// キーボード: R
pub const KEY_R: u32 = 82;
/// キーボード: S
pub const KEY_S: u32 = 83;
/// キーボード: T
pub const KEY_T: u32 = 84;
/// キーボード: U
pub const KEY_U: u32 = 85;
/// キーボード: V
pub const KEY_V: u32 = 86;
/// キーボード: W
pub const KEY_W: u32 = 87;
/// キーボード: X
pub const KEY_X: u32 = 88;
/// キーボード: Y
pub const KEY_Y: u32 = 89;
/// キーボード: Z
pub const KEY_Z: u32 = 90;

/// キーボード: 0
pub const KEY_0: u32 = 48;
/// キーボード: 1
pub const KEY_1: u32 = 49;
/// キーボード: 2
pub const KEY_2: u32 = 50;
/// キーボード: 3
pub const KEY_3: u32 = 51;
/// キーボード: 4
pub const KEY_4: u32 = 52;
/// キーボード: 5
pub const KEY_5: u32 = 53;
/// キーボード: 6
pub const KEY_6: u32 = 54;
/// キーボード: 7
pub const KEY_7: u32 = 55;
/// キーボード: 8
pub const KEY_8: u32 = 56;
/// キーボード: 9
pub const KEY_9: u32 = 57;

/// キーボード: テンキー0
pub const KEY_NUMPAD_0: u32 = 96;
/// キーボード: テンキー1
pub const KEY_NUMPAD_1: u32 = 97;
/// キーボード: テンキー2
pub const KEY_NUMPAD_2: u32 = 98;
/// キーボード: テンキー3
pub const KEY_NUMPAD_3: u32 = 99;
/// キーボード: テンキー4
pub const KEY_NUMPAD_4: u32 = 100;
/// キーボード: テンキー5
pub const KEY_NUMPAD_5: u32 = 101;
/// キーボード: テンキー6
pub const KEY_NUMPAD_6: u32 = 102;
/// キーボード: テンキー7
pub const KEY_NUMPAD_7: u32 = 103;
/// キーボード: テンキー8
pub const KEY_NUMPAD_8: u32 = 104;
/// キーボード: テンキー9
pub const KEY_NUMPAD_9: u32 = 105;

/// キーボード: 上矢印
pub const KEY_UP: u32 = 38;
/// キーボード: 下矢印
pub const KEY_DOWN: u32 = 40;
/// キーボード: 左矢印
pub const KEY_LEFT: u32 = 37;
/// キーボード: 右矢印
pub const KEY_RIGHT: u32 = 39;

/// キーボード: スペース
pub const KEY_SPACE: u32 = 32;
/// キーボード: BackSpace
pub const KEY_BACKSPACE: u32 = 8;
/// キーボード: Tab
pub const KEY_TAB: u32 = 9;
/// キーボード: Enter
pub const KEY_ENTER: u32 = 13;
/// キーボード: Shift
pub const KEY_SHIFT: u32 = 16;
/// キーボード: Ctrl
pub const KEY_CTRL: u32 = 17;
/// キーボード: Alt
pub const KEY_ALT: u32 = 18;
/// キーボード: Pause/Break
pub const KEY_PAUSE: u32 = 19;
/// キーボード: Caps Lock
pub const KEY_CAPS_LOCK: u32 = 20;
/// キーボード: Escape
pub const KEY_ESCAPE: u32 = 27;
/// キーボード: Page Up
pub const KEY_PAGE_UP: u32 = 33;
/// キーボード: Page Down
pub const KEY_PAGE_DOWN: u32 = 34;
/// キーボード: End
pub const KEY_END: u32 = 35;
/// キーボード: Home
pub const KEY_HOME: u32 = 36;
/// キーボード: PrintScreen
pub const KEY_PRINT_SCREEN: u32 = 44;
/// キーボード: Insert
pub const KEY_INSERT: u32 = 45;
/// キーボード: Delete
pub const KEY_DELETE: u32 = 46;

/// キーボード: F1
pub const KEY_F1: u32 = 112;
/// キーボード: F2
pub const KEY_F2: u32 = 113;
/// キーボード: F3
pub const KEY_F3: u32 = 114;
/// キーボード: F4
pub const KEY_F4: u32 = 115;
/// キーボード: F5
pub const KEY_F5: u32 = 116;
/// キーボード: F6
pub const KEY_F6: u32 = 117;
/// キーボード: F7
pub const KEY_F7: u32 = 118;
/// キーボード: F8
pub const KEY_F8: u32 = 119;
/// キーボード: F9
pub const KEY_F9: u32 = 120;
/// キーボード: F10
pub const KEY_F10: u32 = 121;
/// キーボード: F11
pub const KEY_F11: u32 = 122;
/// キーボード: F12
pub const KEY_F12: u32 = 123; 