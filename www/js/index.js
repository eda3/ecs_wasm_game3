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

// オブジェクト参照用の配列
const heap = [];
let heap_next = 0;

// ページ読み込み時の初期化
async function init() {
    try {
        console.log('🔄 Wasmモジュールをロード中...');
        console.log('🔧 ブラウザ情報:', navigator.userAgent);

        // wasm-bindgenが生成したJSモジュールをインポート
        const jsModule = await import('/js/ecs_wasm_game3.js');
        console.log('✅ JSラッパーをロード完了');

        // WASMファイルのパスを指定して初期化関数を実行
        const wasmPath = '/js/ecs_wasm_game3_bg.wasm';
        console.log('🔄 WASM初期化開始... パス:', wasmPath);
        await jsModule.default(wasmPath); // defaultエクスポートを呼び出す
        console.log('✅ WASM初期化完了');

        // モジュールをグローバル変数に保存
        gameModule = jsModule;

        console.log('✅ Wasmモジュールのロードと初期化に成功しました');

        // ゲームロガーを初期化 -> wasm-bindgenが自動で行うため不要
        // gameModule.wasm_logger_init();

        console.log('🎮 Wasm Game Module loaded successfully!');

        // キャンバスを設定
        console.log('🖼️ キャンバス設定開始');
        setupCanvas();
        console.log('✅ キャンバス設定完了');

        // イベントリスナーを設定
        setupEventListeners();

        // ゲームインスタンスを初期化
        try {
            console.log('🚀 ゲームインスタンス初期化開始');
            // `initialize_game` はJSラッパーによってエクスポートされるはず
            gameInstance = gameModule.initialize_game('game-canvas');
            console.log('🚀 Game initialized successfully!');

            // ゲームインスタンスのメソッド一覧を表示
            console.log('📋 利用可能なメソッド:', Object.keys(gameModule));
        } catch (initError) {
            console.error('💥 Failed to initialize game instance:', initError);
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
        const debugInfo = document.getElementById('debug-info');
        if (debugInfo) {
            debugInfo.innerHTML = `<p class="error">初期化エラー: ${error.message}</p><pre>${error.stack}</pre>`;
        }
    }
}

// キャンバスのサイズを設定
function setupCanvas() {
    const canvas = document.getElementById('game-canvas');
    if (!canvas) {
        console.error("キャンバス要素が見つかりません: 'game-canvas'");
        return;
    }

    // キャンバスのサイズを調整（レスポンシブ対応のため）
    function resizeCanvas() {
        const container = canvas.parentElement;
        if (!container) {
            console.error("キャンバスの親要素が見つかりません");
            return;
        }
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
    // gameInstanceのチェックはイベントハンドラ内で行う
    const canvas = document.getElementById('game-canvas');
    if (!canvas) return;

    // キーボードイベント
    document.addEventListener('keydown', (event) => {
        if (gameInstance && typeof gameInstance.handle_key_event === 'function') {
            gameInstance.handle_key_event('keydown', event.code);
        } else if (gameModule && typeof gameModule.handle_key_event === 'function') {
            // グローバル関数としてエクスポートされている場合
            gameModule.handle_key_event('keydown', event.code);
        }
    });

    document.addEventListener('keyup', (event) => {
        if (gameInstance && typeof gameInstance.handle_key_event === 'function') {
            gameInstance.handle_key_event('keyup', event.code);
        } else if (gameModule && typeof gameModule.handle_key_event === 'function') {
            gameModule.handle_key_event('keyup', event.code);
        }
    });

    // マウスイベント
    canvas.addEventListener('mousedown', (event) => {
        if (gameInstance && typeof gameInstance.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mousedown', x, y, event.button);
        } else if (gameModule && typeof gameModule.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameModule.handle_mouse_event('mousedown', x, y, event.button);
        }
    });

    canvas.addEventListener('mouseup', (event) => {
        if (gameInstance && typeof gameInstance.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mouseup', x, y, event.button);
        } else if (gameModule && typeof gameModule.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameModule.handle_mouse_event('mouseup', x, y, event.button);
        }
    });

    canvas.addEventListener('mousemove', (event) => {
        if (gameInstance && typeof gameInstance.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameInstance.handle_mouse_event('mousemove', x, y, null);
        } else if (gameModule && typeof gameModule.handle_mouse_event === 'function') {
            const rect = canvas.getBoundingClientRect();
            const x = (event.clientX - rect.left) * (canvas.width / rect.width);
            const y = (event.clientY - rect.top) * (canvas.height / rect.height);
            gameModule.handle_mouse_event('mousemove', x, y, null);
        }
    });

    // サーバー接続ボタン
    const connectBtn = document.getElementById('connect-btn');
    if (connectBtn) {
        connectBtn.addEventListener('click', connectToServer);
    } else {
        console.warn("接続ボタンが見つかりません: 'connect-btn'");
    }
}

// サーバーに接続
function connectToServer() {
    // gameInstanceのチェックは呼び出し元で行う
    if (!gameModule) {
        console.error('❌ ゲームモジュールがまだ初期化されていません！');
        return;
    }

    const serverUrlInput = document.getElementById('server-url');
    let serverUrl = serverUrlInput ? serverUrlInput.value.trim() : '';
    const connectionStatus = document.getElementById('connection-status');

    console.log('🔍 接続処理開始...');
    console.log(`🔍 入力されたURL: "${serverUrl}"`);

    // サーバーURLが空の場合は、デフォルトのサーバーIPを使用
    if (!serverUrl) {
        serverUrl = 'ws://localhost:8101';
        if (serverUrlInput) serverUrlInput.value = serverUrl;
        console.log('🌐 デフォルトのサーバーURLを使用:', serverUrl);
    }

    // ws://またはwss://で始まっていない場合は、ws://を追加
    if (!serverUrl.startsWith('ws://') && !serverUrl.startsWith('wss://')) {
        console.log(`🔧 URLにプロトコルが含まれていません。ws://を追加します: ${serverUrl} → ws://${serverUrl}`);
        serverUrl = 'ws://' + serverUrl;
        if (serverUrlInput) serverUrlInput.value = serverUrl;
    }

    try {
        console.log(`🌐 サーバーに接続を試みています: ${serverUrl}`);

        // Rustコードのconnect_to_server関数が存在するか確認
        let connectFunction = null;
        if (gameInstance && typeof gameInstance.connect_to_server === 'function') {
            connectFunction = gameInstance.connect_to_server.bind(gameInstance);
        } else if (gameModule && typeof gameModule.connect_to_server === 'function') {
            connectFunction = gameModule.connect_to_server;
        }

        if (!connectFunction) {
            console.error('❌ ERROR: connect_to_server関数が存在しません！');
            if (connectionStatus) {
                connectionStatus.textContent = '接続エラー: 機能未実装';
                connectionStatus.classList.remove('connected');
            }
            return;
        }

        // 接続実行
        connectFunction(serverUrl);
        console.log('✅ connect_to_server関数の呼び出しに成功しました');

        // 接続状態の表示を更新
        if (connectionStatus) {
            connectionStatus.textContent = '接続中...';
            console.log('⏳ 接続状態を「接続中...」に更新しました');
        }

        // 接続状態の確認のためのタイムアウト設定（簡略化のため仮）
        // ... (タイムアウト処理は必要に応じて実装)

    } catch (error) {
        console.error('❌ サーバー接続中にエラーが発生しました:', error);
        if (connectionStatus) {
            connectionStatus.textContent = '接続エラー';
            connectionStatus.classList.remove('connected');
        }
    }
}

// ゲームループを開始
function startGameLoop() {
    // gameInstanceのチェックはループ内で行う

    // 前回のループがあれば停止
    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
    }

    // ゲームループ関数
    function gameLoop(timestamp) {
        if (!lastTime) lastTime = timestamp;
        const deltaTime = (timestamp - lastTime) / 1000;
        lastTime = timestamp;

        // FPS計算
        frameCount++;
        timeSinceLastFpsUpdate += deltaTime;
        if (timeSinceLastFpsUpdate >= 0.5) {
            fpsValue = Math.round(frameCount / timeSinceLastFpsUpdate);
            const fpsCounter = document.getElementById('fps-counter');
            if (fpsCounter) fpsCounter.textContent = fpsValue;
            frameCount = 0;
            timeSinceLastFpsUpdate = 0;
        }

        try {
            // ゲーム状態を更新
            let updateFunction = null;
            if (gameInstance && typeof gameInstance.update === 'function') {
                updateFunction = gameInstance.update.bind(gameInstance);
            } else if (gameModule && typeof gameModule.update === 'function') {
                updateFunction = gameModule.update;
            }
            if (updateFunction) updateFunction(deltaTime);

            // 描画処理
            let renderFunction = null;
            if (gameInstance && typeof gameInstance.render === 'function') {
                renderFunction = gameInstance.render.bind(gameInstance);
            } else if (gameModule && typeof gameModule.render === 'function') {
                renderFunction = gameModule.render;
            }
            if (renderFunction) renderFunction();

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