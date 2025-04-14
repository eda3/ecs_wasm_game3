#!/bin/bash

# 色の設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔨 ECS Wasm Game ビルドスクリプト 🔨${NC}"
echo -e "${YELLOW}-------------------------------------${NC}"

# Wasmビルドのために必要なツールを確認
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}❌ エラー: wasm-packがインストールされていません${NC}"
    echo -e "${YELLOW}インストール方法: cargo install wasm-pack${NC}"
    exit 1
fi

# プロジェクトのルートディレクトリに移動
cd "$(dirname "$0")"/..

# カレントディレクトリの確認と表示
CURRENT_DIR=$(pwd)
echo -e "${BLUE}📂 ビルドディレクトリ: ${CURRENT_DIR}${NC}"

# Cargo.tomlの存在確認
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ エラー: Cargo.tomlが見つかりません${NC}"
    echo -e "${YELLOW}カレントディレクトリ: $(pwd)${NC}"
    exit 1
fi

# クリーン処理を追加（既存のビルド成果物を削除）
echo -e "${BLUE}🧹 古いビルド成果物を削除中...${NC}"
if [ -d "www/pkg" ]; then
    rm -rf www/pkg
    echo -e "${GREEN}✅ www/pkg ディレクトリを削除しました${NC}"
fi

# jsディレクトリの古いファイルを削除
echo -e "${BLUE}🧹 古いJSファイルを削除中...${NC}"
if [ -f "www/js/ecs_wasm_game2.js" ]; then
    rm -f www/js/ecs_wasm_game2.js
    echo -e "${GREEN}✅ 古いJSファイルを削除しました${NC}"
fi
if [ -f "www/js/ecs_wasm_game2_bg.wasm" ]; then
    rm -f www/js/ecs_wasm_game2_bg.wasm
    echo -e "${GREEN}✅ 古いWASMファイルを削除しました${NC}"
fi
if [ -f "www/js/ecs_wasm_game2.d.ts" ]; then
    rm -f www/js/ecs_wasm_game2.d.ts
    echo -e "${GREEN}✅ 古い型定義ファイルを削除しました${NC}"
fi
if [ -f "www/js/ecs_wasm_game2_bg.wasm.d.ts" ]; then
    rm -f www/js/ecs_wasm_game2_bg.wasm.d.ts
    echo -e "${GREEN}✅ 古いWASM型定義ファイルを削除しました${NC}"
fi

echo -e "${BLUE}📦 Wasmパッケージのビルド中...${NC}"

# wasm-packを使用してWebAssemblyパッケージをビルド - バージョンを指定
wasm-pack build --target web --out-dir www/pkg

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Wasmビルドに失敗しました${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Wasmパッケージのビルドが完了しました${NC}"

# JavaScriptの生成されたファイルをwww/jsにコピー
echo -e "${BLUE}📄 JavaScriptファイルをコピー中...${NC}"

# プロジェクト名の確認
if [ ! -f "www/pkg/ecs_wasm_game3.js" ]; then
    echo -e "${RED}❌ 警告: ecs_wasm_game3.jsが見つかりません。パッケージ名を確認します...${NC}"
    PKG_JS=$(find www/pkg -name "*.js" ! -name "*_bg.js" | head -1)
    if [ -z "$PKG_JS" ]; then
        echo -e "${RED}❌ エラー: JSファイルが見つかりません${NC}"
        exit 1
    fi
    PKG_NAME=$(basename "$PKG_JS" .js)
    echo -e "${YELLOW}⚠️ 代わりに${PKG_NAME}.jsを使用します${NC}"
    cp "$PKG_JS" www/js/
    cp "www/pkg/${PKG_NAME}_bg.wasm" www/js/
else
    # Cargo.tomlから正しいパッケージ名を使用（ecs_wasm_game3）
    cp www/pkg/ecs_wasm_game3.js www/js/
    cp www/pkg/ecs_wasm_game3_bg.wasm www/js/
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ ファイルのコピーに失敗しました${NC}"
    exit 1
fi

echo -e "${GREEN}✅ ファイルのコピーが完了しました${NC}"
echo -e "${YELLOW}-------------------------------------${NC}"
echo -e "${GREEN}🎮 ゲームのビルドが完了しました！${NC}"
echo -e "${BLUE}以下のコマンドで開発サーバーを起動します:${NC}"
echo -e "${YELLOW}cd www && npm run dev${NC}"
echo -e "${BLUE}ブラウザで http://162.43.8.148:8001 にアクセスしてください${NC}" 