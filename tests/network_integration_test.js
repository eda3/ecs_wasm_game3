// WebSocketサーバーとクライアントの統合テスト

// テスト設定
const TEST_PORT = 8102;
const MAX_CLIENTS = 3;
const TEST_DURATION = 10000; // テスト実行時間（ミリ秒）

// 必要なモジュールのインポート
const WebSocket = require('ws');
const { spawn } = require('child_process');
const { performance } = require('perf_hooks');

// テスト用サーバー
let server = null;
let serverProcess = null;
let clients = [];
let testResults = {
    connected: 0,
    messagesReceived: 0,
    messageLatency: [],
    errors: [],
};

// メッセージタイプ
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

// サーバーを起動
function startServer() {
    return new Promise((resolve, reject) => {
        console.log(`🚀 起動テスト用WebSocketサーバー (ポート: ${TEST_PORT})...`);

        // サーバープロセスを開始
        serverProcess = spawn('node', ['www/server.js', `--port=${TEST_PORT}`], {
            stdio: 'pipe',
        });

        // 標準出力をキャプチャ
        serverProcess.stdout.on('data', (data) => {
            const output = data.toString();
            console.log(`🖥️ サーバー出力: ${output.trim()}`);

            // サーバーが起動したかどうかをチェック
            if (output.includes('サーバーの準備完了') || output.includes('準備完了')) {
                console.log('✅ サーバーが正常に起動しました');
                resolve();
            }
        });

        // エラー出力をキャプチャ
        serverProcess.stderr.on('data', (data) => {
            const errorOutput = data.toString();
            console.error(`❌ サーバーエラー: ${errorOutput.trim()}`);
            testResults.errors.push(`サーバーエラー: ${errorOutput.trim()}`);
        });

        // サーバープロセスのエラーハンドリング
        serverProcess.on('error', (error) => {
            console.error(`❌ サーバープロセスエラー: ${error.message}`);
            testResults.errors.push(`サーバープロセスエラー: ${error.message}`);
            reject(error);
        });

        // 10秒後にタイムアウト
        setTimeout(() => {
            if (serverProcess) {
                console.error('❌ サーバー起動タイムアウト');
                testResults.errors.push('サーバー起動タイムアウト');
                reject(new Error('サーバー起動タイムアウト'));
            }
        }, 10000);
    });
}

// クライアントを接続
function connectClient(clientId) {
    return new Promise((resolve, reject) => {
        console.log(`👤 クライアント ${clientId} 接続中...`);

        const ws = new WebSocket(`ws://localhost:${TEST_PORT}`);

        // 接続イベント
        ws.on('open', () => {
            console.log(`✅ クライアント ${clientId} 接続成功`);
            testResults.connected++;

            // クライアント情報保存
            clients.push({
                id: clientId,
                socket: ws,
                entityId: null,
                messages: [],
                lastPingTime: null,
            });

            resolve(ws);
        });

        // エラーイベント
        ws.on('error', (error) => {
            console.error(`❌ クライアント ${clientId} 接続エラー: ${error.message}`);
            testResults.errors.push(`クライアント ${clientId} 接続エラー: ${error.message}`);
            reject(error);
        });

        // 5秒後にタイムアウト
        setTimeout(() => {
            if (ws.readyState !== WebSocket.OPEN) {
                console.error(`❌ クライアント ${clientId} 接続タイムアウト`);
                testResults.errors.push(`クライアント ${clientId} 接続タイムアウト`);
                reject(new Error('クライアント接続タイムアウト'));
            }
        }, 5000);
    });
}

// メッセージハンドラを設定
function setupMessageHandler(ws, clientId) {
    const client = clients.find(c => c.id === clientId);
    if (!client) return;

    ws.on('message', (messageData) => {
        try {
            const message = JSON.parse(messageData);
            console.log(`📩 クライアント ${clientId} がメッセージを受信: ${message.message_type}`);

            testResults.messagesReceived++;
            client.messages.push(message);

            // メッセージタイプに応じた処理
            switch (message.message_type) {
                case MessageType.CONNECT_RESPONSE:
                    console.log(`🔗 クライアント ${clientId} の接続が確認されました (プレイヤーID: ${message.player_id})`);
                    break;

                case MessageType.ENTITY_CREATE:
                    console.log(`🎮 クライアント ${clientId} にエンティティ ${message.entity_id} が作成されました`);

                    // 自分のエンティティを追跡
                    if (!client.entityId && message.sequence === 2) { // シーケンス2は自分のエンティティ
                        client.entityId = message.entity_id;
                        console.log(`🎯 クライアント ${clientId} の制御エンティティ: ${message.entity_id}`);
                    }
                    break;

                case MessageType.COMPONENT_UPDATE:
                    // コンポーネント更新の処理
                    console.log(`🔄 エンティティ ${message.entity_id} のコンポーネント更新`);
                    break;

                case MessageType.PONG:
                    // Pingレイテンシを計算
                    if (client.lastPingTime) {
                        const latency = performance.now() - client.lastPingTime;
                        testResults.messageLatency.push(latency);
                        console.log(`⏱️ クライアント ${clientId} のPingレイテンシ: ${latency.toFixed(2)}ms`);
                    }
                    break;
            }
        } catch (error) {
            console.error(`❌ メッセージ解析エラー: ${error.message}`);
            testResults.errors.push(`メッセージ解析エラー: ${error.message}`);
        }
    });
}

// Pingを送信
function sendPing(clientId) {
    const client = clients.find(c => c.id === clientId);
    if (!client || client.socket.readyState !== WebSocket.OPEN) return;

    client.lastPingTime = performance.now();
    const pingMessage = {
        message_type: MessageType.PING,
        sequence: Math.floor(Math.random() * 1000),
        timestamp: Date.now(),
        client_time: Date.now()
    };

    client.socket.send(JSON.stringify(pingMessage));
    console.log(`📤 クライアント ${clientId} がPingを送信`);
}

// 入力を送信
function sendInput(clientId, moveX, moveY) {
    const client = clients.find(c => c.id === clientId);
    if (!client || client.socket.readyState !== WebSocket.OPEN) return;

    // 入力メッセージを作成
    const inputMessage = {
        message_type: MessageType.INPUT,
        sequence: Math.floor(Math.random() * 1000),
        timestamp: Date.now(),
        input_data: {
            movement: [moveX, moveY],
            actions: {},
            timestamp: Date.now()
        }
    };

    client.socket.send(JSON.stringify(inputMessage));
    console.log(`🎮 クライアント ${clientId} が入力を送信: (${moveX}, ${moveY})`);
}

// テスト結果をレポート
function reportResults() {
    console.log('\n====== テスト結果 ======');
    console.log(`接続成功: ${testResults.connected}/${MAX_CLIENTS}`);
    console.log(`受信メッセージ: ${testResults.messagesReceived}`);

    // レイテンシ統計
    if (testResults.messageLatency.length > 0) {
        const avgLatency = testResults.messageLatency.reduce((a, b) => a + b, 0) / testResults.messageLatency.length;
        const minLatency = Math.min(...testResults.messageLatency);
        const maxLatency = Math.max(...testResults.messageLatency);

        console.log(`レイテンシ統計 (ms):`);
        console.log(`  平均: ${avgLatency.toFixed(2)}`);
        console.log(`  最小: ${minLatency.toFixed(2)}`);
        console.log(`  最大: ${maxLatency.toFixed(2)}`);
    }

    // エラー
    if (testResults.errors.length > 0) {
        console.log('\nエラー:');
        testResults.errors.forEach((error, i) => {
            console.log(`  ${i + 1}. ${error}`);
        });
    } else {
        console.log('\n✅ エラーなし');
    }

    console.log('========================\n');
}

// リソースのクリーンアップ
function cleanup() {
    console.log('🧹 リソースをクリーンアップしています...');

    // クライアント接続を閉じる
    clients.forEach(client => {
        if (client.socket && client.socket.readyState === WebSocket.OPEN) {
            client.socket.close();
        }
    });

    // サーバープロセスを終了
    if (serverProcess) {
        serverProcess.kill();
        serverProcess = null;
    }
}

// メインテスト実行
async function runTest() {
    try {
        // サーバーを起動
        await startServer();

        // クライアントを接続
        for (let i = 0; i < MAX_CLIENTS; i++) {
            const ws = await connectClient(i + 1);
            setupMessageHandler(ws, i + 1);
        }

        console.log('🎉 すべてのクライアントが接続されました');

        // テストシナリオを実行
        let testTimer = 0;
        const testInterval = setInterval(() => {
            testTimer += 1000;

            // 各クライアントのアクション
            clients.forEach(client => {
                // Pingを送信 (3秒ごと)
                if (testTimer % 3000 === 0) {
                    sendPing(client.id);
                }

                // ランダムな移動入力を送信 (1秒ごと)
                const moveX = (Math.random() * 2 - 1).toFixed(2);
                const moveY = (Math.random() * 2 - 1).toFixed(2);
                sendInput(client.id, parseFloat(moveX), parseFloat(moveY));
            });

            // テスト終了
            if (testTimer >= TEST_DURATION) {
                clearInterval(testInterval);
                reportResults();
                cleanup();
            }
        }, 1000);

    } catch (error) {
        console.error(`❌ テスト実行エラー: ${error.message}`);
        reportResults();
        cleanup();
    }
}

// テスト実行
console.log('==== WebSocketネットワーク統合テスト開始 ====');
runTest(); 