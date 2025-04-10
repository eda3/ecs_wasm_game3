// WebSocketサーバーの実装
const WebSocket = require('ws');

// サーバー設定
const PORT = 8101;
const HOST = '0.0.0.0';  // すべてのネットワークインターフェースでリッスン

// 詳細な起動情報を表示
console.log(`
====================================================
🚀 WebSocketサーバー起動情報
====================================================
ポート番号: ${PORT}
ホスト: ${HOST}
サーバーURL: ws://localhost:${PORT}
外部接続URL: ws://<あなたのIPアドレス>:${PORT}
現在の接続数: 0
====================================================
`);

// 接続中のクライアント
const clients = new Map();
let nextClientId = 1;

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
ヘッダー:
${Object.entries(headers).map(([key, value]) => `  ${key}: ${value}`).join('\n')}
====================================================
`);

    // クライアントIDを割り当て
    const clientId = nextClientId++;

    // クライアント情報を保存
    clients.set(clientId, {
        socket: socket,
        ip: clientIp,
        lastActivity: Date.now()
    });

    console.log(`👋 クライアント #${clientId} が接続しました (${clientIp})`);
    console.log(`👥 現在の接続数: ${clients.size}`);

    // 接続成功メッセージを送信
    socket.send(JSON.stringify({
        type: 'connect',
        clientId: clientId,
        message: '接続成功！サーバーへようこそ！'
    }));

    // すべてのクライアントに新しいプレイヤーの参加を通知
    broadcastToAll({
        type: 'playerJoined',
        clientId: clientId
    }, clientId);

    // メッセージイベント
    socket.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            console.log(`📩 クライアント #${clientId} からメッセージ:`, data.type || 'unknown');

            // クライアントのアクティビティ時間を更新
            clients.get(clientId).lastActivity = Date.now();

            // メッセージタイプに応じた処理
            switch (data.type) {
                case 'chat':
                    // チャットメッセージを全員に転送
                    broadcastToAll({
                        type: 'chat',
                        clientId: clientId,
                        message: data.message,
                        timestamp: Date.now()
                    });
                    break;

                case 'position':
                    // プレイヤーの位置情報を他のクライアントに転送
                    broadcastToAll({
                        type: 'playerPosition',
                        clientId: clientId,
                        x: data.x,
                        y: data.y,
                        vx: data.vx,
                        vy: data.vy
                    }, clientId);
                    break;

                case 'ping':
                    // Pingには即座にPongで応答
                    socket.send(JSON.stringify({
                        type: 'pong',
                        timestamp: Date.now()
                    }));
                    break;

                default:
                    // 未知のメッセージタイプはそのまま全員に転送
                    broadcastToAll(data, clientId);
            }
        } catch (error) {
            console.error(`⚠️ メッセージ処理エラー (クライアント #${clientId}):`, error.message);
        }
    });

    // 切断イベント
    socket.on('close', () => {
        console.log(`👋 クライアント #${clientId} が切断しました`);

        // クライアントリストから削除
        clients.delete(clientId);

        // 他のクライアントに切断を通知
        broadcastToAll({
            type: 'playerLeft',
            clientId: clientId
        });

        console.log(`👥 現在の接続数: ${clients.size}`);
    });

    // エラーイベント
    socket.on('error', (error) => {
        console.error(`⚠️ クライアント #${clientId} でエラー発生:`, error.message);
    });
});

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
            client.socket.terminate();
            clients.delete(clientId);
        }
    });
}, 60000); // 1分ごとにチェック

// シャットダウン処理
function shutdown() {
    console.log('💤 サーバーをシャットダウンしています...');

    // すべてのクライアントに通知してから切断
    broadcastToAll({
        type: 'serverShutdown',
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