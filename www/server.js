// WebSocketサーバーの実装
const WebSocket = require('ws');

// サーバー設定のデフォルト値
const DEFAULT_PORT = 8101;
const HOST = '0.0.0.0';  // すべてのネットワークインターフェースでリッスン

// コマンドライン引数の解析
function parseCommandLineArgs() {
    const args = process.argv.slice(2);
    let port = DEFAULT_PORT;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '--port' || arg === '-p') {
            // 次の引数をポート番号として解釈
            if (i + 1 < args.length) {
                const portArg = parseInt(args[i + 1]);
                if (!isNaN(portArg) && portArg > 0 && portArg < 65536) {
                    port = portArg;
                    i++; // 次の引数をスキップ
                }
            }
        } else if (arg.startsWith('--port=')) {
            // --port=8080 形式
            const portArg = parseInt(arg.split('=')[1]);
            if (!isNaN(portArg) && portArg > 0 && portArg < 65536) {
                port = portArg;
            }
        }
    }

    return { port };
}

// コマンドライン引数の解析
const { port: PORT } = parseCommandLineArgs();

// メッセージタイプの定義
const MessageType = {
    CONNECT: 'Connect',
    CONNECT_RESPONSE: 'ConnectResponse',
    DISCONNECT: 'Disconnect',
    ENTITY_CREATE: 'EntityCreate',
    ENTITY_DELETE: 'EntityDelete',
    COMPONENT_UPDATE: 'ComponentUpdate',
    INPUT: 'Input',
    TIME_SYNC: 'TimeSync',
    PING: 'Ping',
    PONG: 'Pong',
    ERROR: 'Error'
};

// 詳細な起動情報を表示
console.log(`
====================================================
🚀 ECS WebSocket Game サーバー起動情報
====================================================
ポート番号: ${PORT}
ホスト: ${HOST}
サーバーURL: ws://localhost:${PORT}
外部接続URL: ws://<あなたのIPアドレス>:${PORT}
現在の接続数: 0
プロトコルバージョン: 1.0
====================================================
`);

// 接続中のクライアント
const clients = new Map();
let nextClientId = 1;
let nextEntityId = 1;

// エンティティとコンポーネントの追跡
const entities = new Map();

// WebSocketサーバーを作成
const server = new WebSocket.Server({
    host: HOST,
    port: PORT
});

console.log(`🚀 WebSocketサーバーを起動中... ${HOST}:${PORT}`);

// 接続イベント
server.on('connection', (socket, req) => {
    // 接続元の詳細情報を取得
    const clientIp = req.socket.remoteAddress;
    const clientPort = req.socket.remotePort;
    const clientUrl = req.url || '/';
    const headers = req.headers;
    const userAgent = headers['user-agent'] || 'Unknown';

    console.log(`
====================================================
👋 新規クライアント接続
====================================================
IPアドレス: ${clientIp}
ポート: ${clientPort}
URL: ${clientUrl}
ユーザーエージェント: ${userAgent}
====================================================
`);

    // クライアントIDを割り当て
    const clientId = nextClientId++;

    // クライアント情報を保存
    clients.set(clientId, {
        socket: socket,
        ip: clientIp,
        lastActivity: Date.now(),
        entities: new Set()  // このクライアントが所有するエンティティ
    });

    console.log(`👋 クライアント #${clientId} が接続しました (${clientIp})`);
    console.log(`👥 現在の接続数: ${clients.size}`);

    // 接続成功メッセージを送信（新しいプロトコル形式）
    socket.send(JSON.stringify({
        type: MessageType.CONNECT_RESPONSE,
        sequence: 1,
        timestamp: Date.now(),
        player_id: clientId,
        success: true,
        message: '接続成功！サーバーへようこそ！'
    }));

    // このクライアント用のプレイヤーエンティティを作成
    const playerEntityId = nextEntityId++;

    // エンティティ情報を保存
    entities.set(playerEntityId, {
        owner: clientId,
        components: {
            Position: { x: 400, y: 300, z: 0 },
            Velocity: { x: 0, y: 0, z: 0 },
            PlayerInfo: { player_id: clientId, name: `Player${clientId}` }
        }
    });

    // クライアントにエンティティ所有権を関連付け
    clients.get(clientId).entities.add(playerEntityId);

    // クライアントに自分のエンティティ作成を通知
    socket.send(JSON.stringify({
        type: MessageType.ENTITY_CREATE,
        sequence: 2,
        timestamp: Date.now(),
        entity_id: playerEntityId
    }));

    // クライアントにエンティティのコンポーネントを送信
    socket.send(JSON.stringify({
        type: MessageType.COMPONENT_UPDATE,
        sequence: 3,
        timestamp: Date.now(),
        entity_id: playerEntityId,
        components: entities.get(playerEntityId).components
    }));

    // 他のクライアントに新しいプレイヤーの参加を通知
    broadcastToAll({
        type: MessageType.ENTITY_CREATE,
        sequence: 4,
        timestamp: Date.now(),
        entity_id: playerEntityId
    }, clientId);

    // 他のクライアントに新しいプレイヤーのコンポーネント情報を送信
    broadcastToAll({
        type: MessageType.COMPONENT_UPDATE,
        sequence: 5,
        timestamp: Date.now(),
        entity_id: playerEntityId,
        components: entities.get(playerEntityId).components
    }, clientId);

    // 既存のエンティティ情報を新しいクライアントに送信
    for (const [entityId, entityData] of entities.entries()) {
        // 自分のエンティティは既に通知済みのためスキップ
        if (entityId === playerEntityId) continue;

        // エンティティ作成を通知
        socket.send(JSON.stringify({
            type: MessageType.ENTITY_CREATE,
            sequence: nextSequence(),
            timestamp: Date.now(),
            entity_id: entityId
        }));

        // コンポーネント情報を送信
        socket.send(JSON.stringify({
            type: MessageType.COMPONENT_UPDATE,
            sequence: nextSequence(),
            timestamp: Date.now(),
            entity_id: entityId,
            components: entityData.components
        }));
    }

    // メッセージイベント
    socket.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            const messageType = data.type || data.message_type || 'unknown';
            console.log(`📩 クライアント #${clientId} からメッセージ:`, messageType);

            // クライアントのアクティビティ時間を更新
            clients.get(clientId).lastActivity = Date.now();

            // メッセージタイプに応じた処理
            switch (messageType) {
                case MessageType.INPUT:
                    // 入力データを処理
                    handleInputMessage(clientId, data);
                    break;

                case MessageType.PING:
                    // Pingには即座にPongで応答
                    socket.send(JSON.stringify({
                        type: MessageType.PONG,
                        sequence: nextSequence(),
                        timestamp: Date.now(),
                        client_time: data.client_time,
                        server_time: Date.now()
                    }));
                    break;

                case MessageType.TIME_SYNC:
                    // 時間同期要求に応答
                    socket.send(JSON.stringify({
                        type: MessageType.TIME_SYNC,
                        sequence: nextSequence(),
                        timestamp: Date.now(),
                        client_time: data.client_time,
                        server_time: Date.now()
                    }));
                    break;

                case MessageType.DISCONNECT:
                    // クライアントからの切断通知
                    handleClientDisconnect(clientId, data.reason || 'クライアントリクエスト');
                    break;

                default:
                    console.log(`⚠️ 未処理のメッセージタイプ: ${messageType}`);
            }
        } catch (error) {
            console.error(`⚠️ メッセージ処理エラー (クライアント #${clientId}):`, error.message);
        }
    });

    // 切断イベント
    socket.on('close', () => {
        handleClientDisconnect(clientId, '接続が閉じられました');
    });

    // エラーイベント
    socket.on('error', (error) => {
        console.error(`⚠️ クライアント #${clientId} でエラー発生:`, error.message);
    });
});

// 入力メッセージの処理
function handleInputMessage(clientId, data) {
    const client = clients.get(clientId);
    if (!client) return;

    // クライアントが所有するエンティティを取得
    for (const entityId of client.entities) {
        const entity = entities.get(entityId);
        if (!entity) continue;

        // エンティティの位置を更新（簡易的な例）
        if (data.input_data && entity.components.Position) {
            const input = data.input_data;
            const position = entity.components.Position;
            const velocity = entity.components.Velocity || { x: 0, y: 0, z: 0 };

            // 移動入力に基づいて速度を設定
            if (input.movement) {
                const [moveX, moveY] = input.movement;
                const speed = 5.0; // 移動速度

                velocity.x = moveX * speed;
                velocity.y = moveY * speed;

                // 速度コンポーネントを更新
                entity.components.Velocity = velocity;

                // 位置を更新
                position.x += velocity.x;
                position.y += velocity.y;

                // 画面の境界をチェック（簡易的な実装）
                position.x = Math.max(0, Math.min(800, position.x));
                position.y = Math.max(0, Math.min(600, position.y));
            }

            // 更新されたコンポーネント情報をブロードキャスト
            broadcastToAll({
                type: MessageType.COMPONENT_UPDATE,
                sequence: nextSequence(),
                timestamp: Date.now(),
                entity_id: entityId,
                components: {
                    Position: position,
                    Velocity: velocity
                }
            });
        }
    }
}

// クライアント切断の処理
function handleClientDisconnect(clientId, reason) {
    const client = clients.get(clientId);
    if (!client) return;

    console.log(`👋 クライアント #${clientId} が切断しました: ${reason}`);

    // このクライアントが所有するエンティティを削除
    for (const entityId of client.entities) {
        // 他のクライアントにエンティティ削除を通知
        broadcastToAll({
            type: MessageType.ENTITY_DELETE,
            sequence: nextSequence(),
            timestamp: Date.now(),
            entity_id: entityId
        });

        // エンティティを削除
        entities.delete(entityId);
    }

    // クライアントリストから削除
    clients.delete(clientId);

    console.log(`👥 現在の接続数: ${clients.size}`);
}

// シーケンス番号の生成
let sequenceCounter = 0;
function nextSequence() {
    return ++sequenceCounter;
}

// すべてのクライアントにメッセージをブロードキャスト
function broadcastToAll(data, excludeClientId = null) {
    const message = JSON.stringify(data);

    clients.forEach((client, clientId) => {
        // 除外クライアントIDがあれば、そのクライアントにはメッセージを送らない
        if (excludeClientId !== null && clientId === excludeClientId) {
            return;
        }

        // 接続状態をチェック
        if (client.socket.readyState === WebSocket.OPEN) {
            client.socket.send(message);
        }
    });
}

// 非アクティブなクライアントを定期的にチェックして切断
setInterval(() => {
    const now = Date.now();
    const TIMEOUT = 5 * 60 * 1000; // 5分間無アクティビティのクライアントを切断

    clients.forEach((client, clientId) => {
        if (now - client.lastActivity > TIMEOUT) {
            console.log(`⏰ クライアント #${clientId} は非アクティブのため切断します`);
            handleClientDisconnect(clientId, '非アクティブタイムアウト');
        }
    });
}, 60000); // 1分ごとにチェック

// シャットダウン処理
function shutdown() {
    console.log('💤 サーバーをシャットダウンしています...');

    // すべてのクライアントに通知してから切断
    broadcastToAll({
        type: MessageType.ERROR,
        sequence: nextSequence(),
        timestamp: Date.now(),
        code: 1001,
        message: 'サーバーはシャットダウンします'
    });

    // クライアント接続を閉じる
    clients.forEach((client) => {
        client.socket.close();
    });

    // サーバーを閉じる
    server.close(() => {
        console.log('✅ サーバーが正常にシャットダウンしました');
        process.exit(0);
    });
}

// シグナルハンドラ
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

console.log('✅ WebSocketサーバーの準備完了！接続を待機中...'); 