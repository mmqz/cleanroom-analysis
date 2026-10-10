#!/bin/sh
# push_to_github.sh — 把 acrobat-audit 推上 GitHub(需要 token, 不落盘)
# 用法: GITHUB_TOKEN=ghp_xxx ./push_to_github.sh [github用户名] [仓库名]
# 原则: token 只经环境变量与单次请求头传递, 不写入任何文件/remote URL
set -e
USER="${1:-mmqz}"
REPO="${2:-acrobat-pro-audit}"
TOKEN="${GITHUB_TOKEN:?请先 export GITHUB_TOKEN=你的token}"

HERE=$(cd "$(dirname "$0")/.." && pwd)
cd "$HERE"

echo "[1/3] 创建远端仓库 $USER/$REPO (若已存在则忽略错误)..."
curl -s -o /dev/null -w "%{http_code}\n" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/user/repos \
  -d "{\"name\":\"$REPO\",\"description\":\"Adobe Acrobat Pro DC installer architecture-level security audit (database forensics / install behavior / network surface)\",\"public\":true}" || true

echo "[2/3] 配置 remote(不含token)..."
git remote remove origin 2>/dev/null || true
git remote add origin "https://github.com/$USER/$REPO.git"

echo "[3/3] 推送(token 仅经内存计算 Basic 头, 不持久化)..."
# 本环境 git smart HTTP 端点仅接受 Basic 方案(token/Bearer 均返回 401 invalid credentials)
# base64(user:token) 仅存在于进程内存, 不写入任何文件/remote URL/.git/config
B64=$(printf '%s:%s' "$USER" "$TOKEN" | base64 | tr -d '\n')
git -c credential.helper= -c "http.extraheader=AUTHORIZATION: basic $B64" \
  push -u origin main
unset B64

echo "完成。验证: git ls-remote https://github.com/$USER/$REPO.git"
echo "提醒: 该 token 已在聊天中暴露过, 用毕请立即在 GitHub Settings→Developer settings→Tokens 撤销。"
