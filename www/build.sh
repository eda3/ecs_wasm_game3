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
cd ..

# クリーン処理を追加（既存のビルド成果物を削除）
echo -e "${BLUE}🧹 古いビルド成果物を削除中...${NC}"
if [ -d "www/pkg" ]; then
    rm -rf www/pkg
    echo -e "${GREEN}✅ www/pkg ディレクトリを削除しました${NC}"
fi

echo -e "${BLUE}📦 Wasmパッケージのビルド中...${NC}"

# wasm-packを使用してWebAssemblyパッケージをビルド - バージョンを指定
wasm-pack build --target web --out-dir www/pkg

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Wasmビルドに失敗しました${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Wasmパッケージのビルドが完了しました${NC}"

# JSファイルをコピー
echo -e "${BLUE}📄 JavaScriptファイルをコピー中...${NC}"
cp www/pkg/ecs_wasm_game2.js www/js/
cp www/pkg/ecs_wasm_game2_bg.wasm www/js/

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