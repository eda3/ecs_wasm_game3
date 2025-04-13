# マルチプレイヤーマインスイーパー メッセージプロトコル仕様 📨

## 概要
このドキュメントでは、マルチプレイヤーマインスイーパーのクライアントとサーバー間で使用されるWebSocketメッセージプロトコルを定義します。すべてのメッセージはJSON形式でエンコードされ、特定の構造に従います。

## メッセージ形式

すべてのメッセージは以下の基本構造に従います：

```json
{
  "type": "メッセージタイプ",
  "data": {
    // メッセージ固有のデータフィールド
  }
}
```

- `type`: メッセージの種類を識別する文字列
- `data`: メッセージタイプに関連する追加データを含むオブジェクト

## クライアントからサーバーへのメッセージ

### 1. 接続・ルーム関連

#### 1.1 JoinRoom
プレイヤーが既存のルームに参加するためのリクエスト
```json
{
  "type": "JoinRoom",
  "data": {
    "roomId": "ルームID（例: 「R12345」）",
    "playerName": "プレイヤー名"
  }
}
```

#### 1.2 CreateRoom
新しいゲームルームを作成するためのリクエスト
```json
{
  "type": "CreateRoom",
  "data": {
    "playerName": "プレイヤー名",
    "gameMode": "cooperative" | "competitive",
    "difficulty": "easy" | "medium" | "hard" | "custom",
    "customSettings": {
      "width": 16,
      "height": 16,
      "mines": 40
    }
  }
}
```

#### 1.3 LeaveRoom
プレイヤーがルームから退出するためのリクエスト
```json
{
  "type": "LeaveRoom",
  "data": {}
}
```

#### 1.4 StartGame
ゲームを開始するためのリクエスト（ルームホストのみ）
```json
{
  "type": "StartGame",
  "data": {}
}
```

#### 1.5 Ping
接続維持のためのpingメッセージ
```json
{
  "type": "Ping",
  "data": {
    "timestamp": 1682312345678
  }
}
```

### 2. ゲームプレイ関連

#### 2.1 RevealCell
セルを開く操作
```json
{
  "type": "RevealCell",
  "data": {
    "x": 5,
    "y": 7
  }
}
```

#### 2.2 ToggleFlag
セルにフラグを立てる/取り除く操作
```json
{
  "type": "ToggleFlag",
  "data": {
    "x": 8,
    "y": 3
  }
}
```

#### 2.3 ChordAction
ナンバーセル周囲の未開封セルを一度に開く操作（周囲のフラグ数が数字と一致する場合）
```json
{
  "type": "ChordAction",
  "data": {
    "x": 4,
    "y": 6
  }
}
```

### 3. コミュニケーション関連

#### 3.1 ChatMessage
チャットメッセージの送信
```json
{
  "type": "ChatMessage",
  "data": {
    "message": "こんにちは！"
  }
}
```

#### 3.2 PlayerReady
プレイヤーがゲーム開始準備完了を通知
```json
{
  "type": "PlayerReady",
  "data": {
    "ready": true
  }
}
```

## サーバーからクライアントへのメッセージ

### 1. 接続・ルーム関連

#### 1.1 Welcome
接続成功時の初期メッセージ
```json
{
  "type": "Welcome",
  "data": {
    "playerId": "P12345",
    "serverTime": 1682312345678
  }
}
```

#### 1.2 RoomJoined
ルーム参加成功時のメッセージ
```json
{
  "type": "RoomJoined",
  "data": {
    "roomId": "R12345",
    "gameMode": "cooperative" | "competitive",
    "difficulty": "easy" | "medium" | "hard" | "custom",
    "customSettings": {
      "width": 16,
      "height": 16,
      "mines": 40
    },
    "players": [
      {
        "id": "P12345",
        "name": "プレイヤー1",
        "isHost": true,
        "ready": false
      },
      {
        "id": "P67890",
        "name": "プレイヤー2",
        "isHost": false,
        "ready": true
      }
    ]
  }
}
```

#### 1.3 RoomCreated
ルーム作成成功時のメッセージ
```json
{
  "type": "RoomCreated",
  "data": {
    "roomId": "R12345",
    "gameMode": "cooperative" | "competitive",
    "difficulty": "easy" | "medium" | "hard" | "custom",
    "customSettings": {
      "width": 16,
      "height": 16,
      "mines": 40
    }
  }
}
```

#### 1.4 PlayerJoined
新しいプレイヤーがルームに参加した時のメッセージ
```json
{
  "type": "PlayerJoined",
  "data": {
    "player": {
      "id": "P67890",
      "name": "プレイヤー2",
      "isHost": false,
      "ready": false
    }
  }
}
```

#### 1.5 PlayerLeft
プレイヤーがルームを退出した時のメッセージ
```json
{
  "type": "PlayerLeft",
  "data": {
    "playerId": "P67890"
  }
}
```

#### 1.6 GameStarted
ゲームが開始された時のメッセージ
```json
{
  "type": "GameStarted",
  "data": {
    "boardId": "B12345",
    "startTime": 1682312345678,
    "board": {
      "width": 16,
      "height": 16,
      "mineCount": 40,
      // オプション：協力モードではクライアント側で生成せず、サーバーから初期ボードをロードする場合もある
      "cells": [
        [0, 0, 0, 1, ...],
        [0, 0, 1, 2, ...],
        ...
      ]
    }
  }
}
```

#### 1.7 Pong
Pingに対する応答
```json
{
  "type": "Pong",
  "data": {
    "timestamp": 1682312345678,
    "serverTime": 1682312345700
  }
}
```

### 2. ゲームプレイ関連

#### 2.1 CellRevealed
セルが開かれたことを通知
```json
{
  "type": "CellRevealed",
  "data": {
    "playerId": "P12345",
    "x": 5,
    "y": 7,
    "value": 3, // セルの値: 0-8の数字、-1で地雷
    "revealedCells": [
      {"x": 5, "y": 7, "value": 3},
      // 空白セルの場合、連鎖的に開かれたセルのリスト
    ]
  }
}
```

#### 2.2 FlagToggled
フラグがトグルされたことを通知
```json
{
  "type": "FlagToggled",
  "data": {
    "playerId": "P12345",
    "x": 8,
    "y": 3,
    "isFlagged": true
  }
}
```

#### 2.3 ChordPerformed
コード操作が実行されたことを通知
```json
{
  "type": "ChordPerformed",
  "data": {
    "playerId": "P12345",
    "x": 4,
    "y": 6,
    "revealedCells": [
      {"x": 3, "y": 5, "value": 1},
      {"x": 3, "y": 6, "value": 2},
      // コードによって開かれたセルのリスト
    ]
  }
}
```

#### 2.4 GameOver
ゲーム終了を通知（負け）
```json
{
  "type": "GameOver",
  "data": {
    "result": "defeat",
    "causePlayerId": "P12345", // 負けの原因となったプレイヤー
    "mineLocation": {"x": 5, "y": 7}, // 爆発した地雷の位置
    "allMines": [
      {"x": 1, "y": 2},
      {"x": 5, "y": 7},
      // すべての地雷の位置
    ],
    "scores": [
      {"playerId": "P12345", "name": "プレイヤー1", "score": 42},
      {"playerId": "P67890", "name": "プレイヤー2", "score": 31}
    ],
    "gameTime": 187 // ゲームにかかった秒数
  }
}
```

#### 2.5 GameWon
ゲーム終了を通知（勝ち）
```json
{
  "type": "GameWon",
  "data": {
    "scores": [
      {"playerId": "P12345", "name": "プレイヤー1", "score": 125},
      {"playerId": "P67890", "name": "プレイヤー2", "score": 115}
    ],
    "gameTime": 253, // ゲームにかかった秒数
    "winner": "P12345" // 競争モードでのみ。協力モードでは省略可
  }
}
```

### 3. コミュニケーション関連

#### 3.1 ChatReceived
チャットメッセージを受信
```json
{
  "type": "ChatReceived",
  "data": {
    "playerId": "P12345",
    "playerName": "プレイヤー1",
    "message": "こんにちは！",
    "timestamp": 1682312345678
  }
}
```

#### 3.2 PlayerReadyChanged
プレイヤーの準備状態が変更された
```json
{
  "type": "PlayerReadyChanged",
  "data": {
    "playerId": "P12345",
    "ready": true
  }
}
```

### 4. システムメッセージ

#### 4.1 Error
エラーメッセージ
```json
{
  "type": "Error",
  "data": {
    "code": "ROOM_NOT_FOUND",
    "message": "指定されたルームが見つかりませんでした"
  }
}
```

#### 4.2 SystemMessage
システムからの通知
```json
{
  "type": "SystemMessage",
  "data": {
    "messageType": "INFO" | "WARNING",
    "message": "30秒後にサーバーメンテナンスが始まります",
    "timestamp": 1682312345678
  }
}
```

## エラーコード一覧

| コード | 説明 |
|--------|------|
| INVALID_MESSAGE | 不正なメッセージ形式 |
| ROOM_NOT_FOUND | 指定されたルームが存在しない |
| ROOM_FULL | ルームが満員 |
| GAME_ALREADY_STARTED | ゲームはすでに開始している |
| NOT_ROOM_HOST | ルームホストでないプレイヤーが権限操作を試みた |
| INVALID_MOVE | 不正なゲーム操作 |
| NAME_ALREADY_TAKEN | 同じ名前のプレイヤーが既に存在する |
| INVALID_NAME | 無効なプレイヤー名 |
| SERVER_ERROR | サーバー内部エラー |

## メッセージフロー例

### ゲームルーム作成と参加の例

1. クライアント → サーバー: `JoinRoom` または `CreateRoom`
2. サーバー → クライアント: `Welcome`
3. サーバー → クライアント: `RoomJoined` または `RoomCreated`
4. サーバー → 他のクライアント: `PlayerJoined`
5. クライアント → サーバー: `PlayerReady`
6. サーバー → すべてのクライアント: `PlayerReadyChanged`
7. ホスト → サーバー: `StartGame`
8. サーバー → すべてのクライアント: `GameStarted`

### ゲームプレイの例

1. クライアント → サーバー: `RevealCell`
2. サーバー → すべてのクライアント: `CellRevealed`
3. クライアント → サーバー: `ToggleFlag`
4. サーバー → すべてのクライアント: `FlagToggled`
5. クライアント → サーバー: `RevealCell` (地雷の場合)
6. サーバー → すべてのクライアント: `GameOver`

または

5. クライアント → サーバー: `RevealCell` (最後の安全なセルの場合)
6. サーバー → すべてのクライアント: `GameWon`

## 実装上の注意

1. メッセージの検証: サーバーはすべてのメッセージを厳密に検証し、不正なメッセージは拒否すべきです
2. エラー処理: エラー発生時は適切なエラーコードと説明でクライアントに通知する
3. メッセージサイズ: 特に大きなゲームボードでは、メッセージサイズに注意を払う
4. タイムアウト: 長時間応答のないクライアントは切断処理を行う
5. 再接続: クライアントの再接続機能をサポートし、ゲーム状態を適切に復元する

## 将来の拡張性

1. 観戦モード: 進行中のゲームを観戦するための機能
2. カスタムルール: ゲームのバリエーションをサポートするための拡張
3. ユーザー認証: 将来的に永続的なユーザープロファイルをサポートするための拡張

## バージョン履歴

| バージョン | 日付 | 変更内容 |
|------------|------|----------|
| 1.0.0 | 2025-04-13 | 初版作成 | 