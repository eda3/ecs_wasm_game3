// ゲームモジュールとインスタンス
let gameModule = null;
let gameInstance = null;

// FPS計算用の変数
let lastTime = 0;
let fpsUpdateCounter = 0;
let fpsValue = 0;
let frameCount = 0;
let timeSinceLastFpsUpdate = 0;

// アニメーションフレームID
let animationFrameId = null;

// ページ読み込み時の初期化
async function init() {
    try {
        console.log('🔄 Wasmモジュールをロード中...');

        // Wasmモジュールをロード前に環境チェック
        if (typeof window.FinalizationRegistry === 'undefined') {
            console.warn('⚠️ FinalizationRegistryがサポートされていません。ポリフィルを使用します。');
            // 簡易的なポリフィル
            window.FinalizationRegistry = class {
                constructor(callback) { this.callback = callback; }
                register(obj, value) { /* ポリフィル実装 */ }
                unregister(obj) { /* ポリフィル実装 */ }
            };
        }

        // wasm_bindgen内部のためのグローバル関数を追加
        window.__wbg_function_table = [];
        window.__wbindgen_export_2 = { set: function (idx, obj) { window.__wbg_function_table[idx] = obj; } };

        try {
            // Wasmモジュールをロード
            gameModule = await import('./ecs_wasm_game2.js');
            await gameModule.default();
            console.log('✅ Wasmモジュールのロードに成功しました');
        } catch (moduleError) {
            console.error('❌ Wasmモジュールのロード中にエラーが発生しました:', moduleError);

            // エラーメッセージをUI上に表示
            const debugInfo = document.getElementById('debug-info');
            if (debugInfo) {
                debugInfo.innerHTML = `<p class="error">モジュールロードエラー: ${moduleError.message}</p>`;
            }
            throw moduleError;
        }

        // ゲームロガーを初期化
        gameModule.wasm_logger_init();

        console.log('🎮 Wasm Game Module loaded successfully!');

        // ゲームキャンバスを設定
        setupCanvas();

        // イベントリスナーを設定
        setupEventListeners();

        // ゲームインスタンスを初期化
        try {
            gameInstance = gameModule.initialize_game('game-canvas');
            console.log('🚀 Game initialized successfully!');
        } catch (initError) {
            console.error('💥 Failed to initialize game instance:', initError);

            // エラーメッセージをUI上に表示
            const debugInfo = document.getElementById('debug-info');
            if (debugInfo) {
                debugInfo.innerHTML = `<p class="error">初期化エラー: ${initError.message}</p>`;
            }
            throw initError;
        }

        // ゲームループを開始
        startGameLoop();

        // ゲーム初期化後に自動的にサーバーに接続
        setTimeout(() => {
            console.log('🌐 Auto-connecting to server...');
            connectToServer();
        }, 500);
    } catch (error) {
        console.error('💥 Failed to initialize game:', error);
    }
}

// キャンバスのサイズを設定
function setupCanvas() {
    const canvas = document.getElementById('game-canvas');

    // キャンバスのサイズを調整（レスポンシブ対応のため）
    function resizeCanvas() {
        const container = canvas.parentElement;
        const containerWidth = container.clientWidth;

        // アスペクト比を維持
        const aspectRatio = 800 / 600;

        // コンテナ幅に合わせてキャンバスのサイズを設定
        canvas.width = containerWidth;
        canvas.height = containerWidth / aspectRatio;
    }

    // 初期サイズ設定
    resizeCanvas();

    // ウィンドウリサイズ時にキャンバスサイズを調整
    window.addEventListener('resize', resizeCanvas);
}

// イベントリスナーの設定
function setupEventListeners() {
    if (!gameInstance) return;

    const canvas = document.getElementById('game-canvas');

    // キーボードイベント
    document.addEventListener('keydown', (event) => {
        if (gameInstance) {
            gameInstance.handle_key_event('keydown', event.code);
        }
    });

    document.addEventListener('keyup', (event) => {
        if (gameInstance) {
            gameInstance.handle_key_event('keyup', event.code);
        }
    });

    // マウスイベント
    canvas.addEventListener('mousedown', (event) => {
        if (gameInstance) {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mousedown', x, y, event.button);
        }
    });

    canvas.addEventListener('mouseup', (event) => {
        if (gameInstance) {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mouseup', x, y, event.button);
        }
    });

    canvas.addEventListener('mousemove', (event) => {
        if (gameInstance) {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mousemove', x, y, null);
        }
    });

    // サーバー接続ボタン
    const connectBtn = document.getElementById('connect-btn');
    connectBtn.addEventListener('click', connectToServer);
}

// サーバーに接続
function connectToServer() {
    if (!gameInstance) {
        console.error('❌ ゲームインスタンスがまだ初期化されていません！');
        return;
    }

    const serverUrlInput = document.getElementById('server-url');
    let serverUrl = serverUrlInput.value.trim();
    const connectionStatus = document.getElementById('connection-status');

    console.log('🔍 接続処理開始...');
    console.log(`🔍 入力されたURL: "${serverUrl}"`);

    // サーバーURLが空の場合は、デフォルトのサーバーIPを使用
    if (!serverUrl) {
        // localhost:8101をデフォルトに設定（サーバーが同じホストで動いている場合）
        serverUrl = 'ws://localhost:8101';
        serverUrlInput.value = serverUrl; // 入力欄にもデフォルト値を表示
        console.log('🌐 デフォルトのサーバーURLを使用:', serverUrl);
    }

    // ws://またはwss://で始まっていない場合は、ws://を追加
    if (!serverUrl.startsWith('ws://') && !serverUrl.startsWith('wss://')) {
        console.log(`🔧 URLにプロトコルが含まれていません。ws://を追加します: ${serverUrl} → ws://${serverUrl}`);
        serverUrl = 'ws://' + serverUrl;
        serverUrlInput.value = serverUrl;
    }

    try {
        console.log(`🌐 サーバーに接続を試みています: ${serverUrl}`);

        // Rustコードのconnect_to_server関数が存在するか確認
        if (typeof gameInstance.connect_to_server !== 'function') {
            console.error('❌ ERROR: gameInstance.connect_to_server関数が存在しません！');
            connectionStatus.textContent = '接続エラー: 機能未実装';
            connectionStatus.classList.remove('connected');
            return;
        }

        // 接続実行
        try {
            gameInstance.connect_to_server(serverUrl);
            console.log('✅ connect_to_server関数の呼び出しに成功しました');
        } catch (callError) {
            console.error('❌ connect_to_server関数の呼び出し中にエラーが発生:', callError);
            throw callError;
        }

        // 接続状態の表示を更新
        connectionStatus.textContent = '接続中...';
        console.log('⏳ 接続状態を「接続中...」に更新しました');

        // 接続状態の確認のためのタイムアウト設定
        const connectionTimeout = setTimeout(() => {
            // 5秒後も接続状態が変わらない場合はタイムアウト
            if (connectionStatus.textContent === '接続中...') {
                connectionStatus.textContent = 'タイムアウト';
                connectionStatus.classList.remove('connected');
                console.error('⏱️ 接続がタイムアウトしました。サーバーが実行中か確認してください。');
            }
        }, 5000);

        // 本来はサーバーからの接続成功応答に基づいて表示を変更すべき
        // 仮の実装として1秒後に接続成功と表示
        setTimeout(() => {
            // まだ接続中の場合のみ成功としてマーク
            if (connectionStatus.textContent === '接続中...') {
                clearTimeout(connectionTimeout); // タイムアウトタイマーをクリア
                connectionStatus.textContent = '接続済み';
                connectionStatus.classList.add('connected');
                console.log('✅ 接続に成功したとみなします（仮の実装）');
            }
        }, 1000);
    } catch (error) {
        console.error('❌ サーバー接続中にエラーが発生しました:', error);
        connectionStatus.textContent = '接続エラー';
        connectionStatus.classList.remove('connected');
    }
}

// ゲームループを開始
function startGameLoop() {
    if (!gameInstance) return;

    // 前回のループがあれば停止
    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
    }

    // ゲームループ関数
    function gameLoop(timestamp) {
        if (!lastTime) lastTime = timestamp;

        // 前フレームからの経過時間（秒）
        const deltaTime = (timestamp - lastTime) / 1000;
        lastTime = timestamp;

        // FPS計算
        frameCount++;
        timeSinceLastFpsUpdate += deltaTime;

        // 0.5秒ごとにFPS表示を更新
        if (timeSinceLastFpsUpdate >= 0.5) {
            fpsValue = Math.round(frameCount / timeSinceLastFpsUpdate);
            document.getElementById('fps-counter').textContent = fpsValue;
            frameCount = 0;
            timeSinceLastFpsUpdate = 0;
        }

        try {
            // ゲーム状態を更新
            gameInstance.update(deltaTime);

            // 描画処理
            gameInstance.render();

            // 次のフレームをリクエスト
            animationFrameId = requestAnimationFrame(gameLoop);
        } catch (error) {
            console.error('💥 Game loop error:', error);
            cancelAnimationFrame(animationFrameId);
        }
    }

    // 最初のフレームをリクエスト
    animationFrameId = requestAnimationFrame(gameLoop);
}

// ページ読み込み完了時に初期化
window.addEventListener('load', init); 