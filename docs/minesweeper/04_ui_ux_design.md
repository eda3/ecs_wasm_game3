# ゲームフレームワーク UI/UX設計書

## 概要

このドキュメントでは、汎用ゲームフレームワークのユーザーインターフェースとユーザーエクスペリエンスの設計指針について説明します。このフレームワークは様々なゲームに対応できるよう、柔軟かつ直感的なUIを提供することを目指しています。

## デザイン原則

1. **シンプルさ**：不必要な要素を排除し、ユーザーが混乱しないようにする
2. **一貫性**：同じ機能には同じUIパターンを使用し、学習曲線を緩やかにする
3. **レスポンシブ**：様々な画面サイズに対応するレイアウト
4. **アクセシビリティ**：多様なユーザーが利用できるよう配慮する
5. **フィードバック**：ユーザーのアクションに対して明確なフィードバックを提供する

## 画面構成

### 共通レイアウト

すべての画面は以下の基本レイアウトに従います：

```
+------------------------------------------+
|              ヘッダー部分                 |
|  (ロゴ、ナビゲーション、ユーザー情報など)  |
+------------------------------------------+
|                                          |
|                                          |
|              メインコンテンツ             |
|                                          |
|                                          |
+------------------------------------------+
|               フッター部分                |
|        (著作権情報、リンクなど)           |
+------------------------------------------+
```

### メイン画面

```
+------------------------------------------+
|  ロゴ            ユーザー名    設定      |
+------------------------------------------+
|                                          |
|  +-------------+      +-------------+    |
|  | ゲーム作成   |      | ゲーム参加   |    |
|  +-------------+      +-------------+    |
|                                          |
|  +------------------------------------+  |
|  |         アクティブなゲーム         |  |
|  |  • ゲームルーム1 (2/4人)          |  |
|  |  • ゲームルーム2 (3/8人)          |  |
|  |  ...                              |  |
|  +------------------------------------+  |
|                                          |
+------------------------------------------+
|          © 2023 ゲームフレームワーク      |
+------------------------------------------+
```

### ルーム作成画面

```
+------------------------------------------+
|  ロゴ          ルーム作成    戻る        |
+------------------------------------------+
|                                          |
|  ルーム名: [                    ]        |
|                                          |
|  ゲームタイプ:                           |
|  ○ タイプA  ○ タイプB  ○ タイプC        |
|                                          |
|  プレイヤー数: [ - 4 + ]                 |
|                                          |
|  ゲームモード:                           |
|  ○ 協力  ○ 対戦  ○ チーム               |
|                                          |
|  詳細設定:                               |
|  □ カスタム設定1                         |
|  □ カスタム設定2                         |
|                                          |
|  [       ルームを作成       ]            |
|                                          |
+------------------------------------------+
|          © 2023 ゲームフレームワーク      |
+------------------------------------------+
```

### ロビー画面

```
+------------------------------------------+
|  ルーム: ABC123        ユーザー名  退出  |
+------------------------------------------+
|                                          |
|  +-----------------+  +--------------+   |
|  | プレイヤーリスト |  |    チャット   |   |
|  | • プレイヤー1 ✓  |  |              |   |
|  | • プレイヤー2    |  | P1: こんにちは |   |
|  | • プレイヤー3 ✓  |  | P2: よろしく  |   |
|  | • [空き]        |  |              |   |
|  +-----------------+  |              |   |
|                       |              |   |
|  ゲーム設定:           |              |   |
|  - タイプ: タイプA     | [メッセージ]  |   |
|  - モード: 協力        | [   送信   ]  |   |
|  - 詳細: カスタム設定1 +--------------+   |
|                                          |
|  [   準備完了   ]    [   ゲーム開始   ]  |
|                                          |
+------------------------------------------+
|          © 2023 ゲームフレームワーク      |
+------------------------------------------+
```

### ゲームプレイ画面

```
+------------------------------------------+
|  ルーム: ABC123     時間: 03:45   退出   |
+------------------------------------------+
|  プレイヤー情報:                         |
|  P1: 100pt  P2: 85pt  P3: 120pt         |
+------------------------------------------+
|                                          |
|                                          |
|                                          |
|             ゲームコンテンツ             |
|            (ゲーム固有の表示)            |
|                                          |
|                                          |
|                                          |
+------------------------------------------+
|  アクション:  [A] [B] [C]    チャット ▼  |
+------------------------------------------+
|          © 2023 ゲームフレームワーク      |
+------------------------------------------+
```

### ゲーム結果画面

```
+------------------------------------------+
|  ルーム: ABC123       結果      ホームへ |
+------------------------------------------+
|                                          |
|           ゲーム結果: 勝利/敗北           |
|                                          |
|  +------------------------------------+  |
|  |           最終スコア              |  |
|  |  1. プレイヤー3 - 120pt           |  |
|  |  2. プレイヤー1 - 100pt           |  |
|  |  3. プレイヤー2 - 85pt            |  |
|  +------------------------------------+  |
|                                          |
|  ゲーム統計:                             |
|  - プレイ時間: 10:23                     |
|  - アクション数: 142                     |
|  - 特別イベント: 5                       |
|                                          |
|  [  もう一度プレイ  ]  [ ロビーに戻る ]  |
|                                          |
+------------------------------------------+
|          © 2023 ゲームフレームワーク      |
+------------------------------------------+
```

## 色彩計画

### 基本カラーパレット

- **プライマリーカラー**: `#4a6da7` (青) - ヘッダー、ボタン、強調表示
- **セカンダリーカラー**: `#5cb85c` (緑) - 成功、正のフィードバック
- **アクセントカラー**: `#f0ad4e` (オレンジ) - 警告、注意喚起
- **エラーカラー**: `#d9534f` (赤) - エラー、危険、失敗
- **ニュートラルカラー**: 
  - `#ffffff` (白) - 背景
  - `#f8f9fa` (薄い灰色) - セクション背景
  - `#e9ecef` (中間の灰色) - 境界線、分離線
  - `#343a40` (濃い灰色) - 通常テキスト
  - `#212529` (ほぼ黒) - ヘッダーテキスト

### 機能別の色使い

- **ボタン**:
  - 通常: プライマリーカラー
  - 準備完了/開始: セカンダリーカラー
  - キャンセル/退出: エラーカラー
  - 二次的なアクション: ニュートラルカラー

- **ステータス表示**:
  - 準備完了: セカンダリーカラー
  - 準備中: アクセントカラー
  - エラー/切断: エラーカラー
  - 勝利: セカンダリーカラー
  - 敗北: エラーカラー（明度を上げる）

## タイポグラフィ

- **ベースフォント**: 'Noto Sans JP', sans-serif
- **見出し**: 'Roboto', sans-serif

### フォントサイズ

- **大見出し**: 28px
- **中見出し**: 22px
- **小見出し**: 18px
- **本文**: 16px
- **補足情報**: 14px
- **ボタンテキスト**: 16px

## アイコンと視覚要素

- **アイコンセット**: Font Awesome または Material Iconsを統一して使用
- **視覚的階層**: 重要な要素ほど大きく、コントラストを高く

### 共通アイコン

- ホーム: `🏠`
- 設定: `⚙️`
- プレイヤー: `👤`
- チャット: `💬`
- 準備完了: `✓`
- 時間: `⏱️`
- スコア: `🏆`
- 退出: `🚪`

## アニメーションとトランジション

### 基本原則

- **目的を持つ**: 単なる装飾ではなく、状態変化や注目の誘導に使用
- **短く滑らか**: 200ms以下の短いアニメーション
- **一貫性**: 同じ操作には同じアニメーションを使用

### 使用箇所

- **画面遷移**: フェード効果（150ms）
- **ボタン**: ホバー時に軽い拡大（110%、100ms）
- **通知**: スライドイン（右から、200ms）
- **モーダル**: フェードイン + スケール（150ms）
- **エラー**: 軽い振動効果（100ms）

## レスポンシブデザイン

### ブレークポイント

- **スマートフォン**: 576px未満
- **タブレット**: 576px〜991px
- **デスクトップ**: 992px以上

### 対応方針

- **スマートフォン**: 
  - 単一カラムレイアウト
  - フォントサイズ縮小（最大15%）
  - タッチ操作に最適化（ボタンサイズ拡大）

- **タブレット**:
  - 2カラムレイアウト
  - チャットとゲーム画面の並列表示

- **デスクトップ**:
  - 最大3カラムレイアウト
  - サイドバーでの追加情報表示

## アクセシビリティ

- **コントラスト比**: WCAG AA基準（4.5:1）以上を確保
- **キーボード操作**: すべての機能をキーボードで操作可能に
- **スクリーンリーダー対応**: 適切なARIA属性の使用
- **フォーカス表示**: フォーカス状態を視覚的に明確に
- **エラーメッセージ**: 明確でアクションにつながる内容

## 状態とフィードバック

### インタラクション状態

- **通常状態**: デフォルト表示
- **ホバー状態**: 軽い色変化とカーソル変更
- **アクティブ状態**: より顕著な色変化と軽い陥没効果
- **無効状態**: 透明度を下げてクリックできないことを示す

### フィードバック方法

- **視覚的フィードバック**: 色、アイコン、アニメーションの変化
- **テキストフィードバック**: 通知、トースト、ツールチップ
- **音声フィードバック**: 重要なアクションや結果に対する短い効果音

## ユーザビリティテスト計画

1. **テスト目的**: UI/UXの使いやすさと直感性の検証
2. **テスト参加者**: 様々な年齢層と経験レベルのユーザー（5〜10名）
3. **テストシナリオ**:
   - ルームの作成と参加
   - ゲーム開始とプレイ
   - チャット機能の利用
   - エラー状態からの回復
4. **評価指標**:
   - タスク完了率
   - エラー発生率
   - 主観的満足度（アンケート）
   - タスク完了までの時間

## 実装ガイドライン

### コンポーネント構造

- **再利用可能コンポーネント**:
  - ボタン
  - 入力フィールド
  - モーダル
  - カード
  - トグルスイッチ
  - ドロップダウン
  - 通知

### CSSアプローチ

- **基本**: CSS Modules または Styled Components
- **命名規則**: BEM（Block, Element, Modifier）
- **ユーティリティクラス**: 共通のマージン、パディング、色などに

### レンダリング最適化

- **コンポーネントの分割**: 適切なサイズに分解
- **条件付きレンダリング**: 必要な部分のみ更新
- **メモ化**: 不必要な再計算を避ける

## プロトタイプとテスト

プロトタイプは以下のツールを使用して作成し、イテレーティブに改善します：

1. **ローファイワイヤーフレーム**: Figma/Adobe XD
2. **インタラクティブプロトタイプ**: Figma/Framer
3. **実装テスト**: 簡易的なHTMLとCSS

## 拡張性とカスタマイズ

フレームワークは以下の方法でカスタマイズ可能です：

1. **テーマ変更**: カラーパレットとフォントの変更
2. **レイアウト調整**: グリッドシステムによるレイアウト調整
3. **カスタムコンポーネント**: 特定のゲーム向けUI要素の追加
4. **アニメーションカスタマイズ**: タイミングと効果のオーバーライド 