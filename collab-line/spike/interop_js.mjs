// 探针 3b 互操作 JS 端（npm yjs 原版 —— 跨语言 wire 兼容的另一半实证）：
// 读 Rust(yrs 0.28) 写出的 update v1 → apply 到 Y.Doc → 断言内容 →
// 追加编辑 → 全量 update 写出 + SV 写出。
import { readFileSync, writeFileSync } from 'node:fs';
import * as Y from 'yjs';

const rustUpdate = readFileSync('out/rust_to_js.bin');
const rustSv = readFileSync('out/rust_sv.bin');

const doc = new Y.Doc();
Y.applyUpdate(doc, rustUpdate);

const text = doc.getText('interop');
const map = doc.getMap('meta');
const arr = doc.getArray('blocks');

// 断言 Rust 侧内容全部可见
const s = text.toString();
if (!s.includes('hello from rust ') || !s.includes('yrs 0.28')) {
  throw new Error(`Rust text 不可见: ${s}`);
}
if (map.get('engine') !== 'yrs' || map.get('answer') !== 42) {
  throw new Error(`Rust map 不可见: ${JSON.stringify([...map.entries()])}`);
}
if (arr.length !== 2 || arr.get(0) !== 'para-1') {
  throw new Error(`Rust array 不可见: ${JSON.stringify(arr.toJSON())}`);
}

// 练习 SV 互通：用 Rust 写出的 SV 求 diff（应恰为空 —— JS 尚无新内容）
const emptyDiff = Y.encodeStateAsUpdate(doc, rustSv);
if (emptyDiff.length === 0) {
  console.log('P3b NOTE: Y.encodeStateAsUpdate(doc, sv) 返回空 diff（SV 过滤生效）');
} else {
  // 非零也可能含序言头，仅登记不判死
  console.log(`P3b NOTE: 定向 update 长度 ${emptyDiff.length}B（含 lib0 序言）`);
}

// JS 侧追加编辑（模拟第二端）
text.insert(0, '[js prefix] ');
map.set('jsKey', 'from-js');

// 全量更新写出（Rust 端将 apply 回验）
const fullUpdate = Y.encodeStateAsUpdate(doc);
writeFileSync('out/js_to_rust.bin', fullUpdate);
writeFileSync('out/js_sv.bin', Y.encodeStateVector(doc));

console.log(`P3b PASS: yjs(npm) apply Rust update 成功；text=${text.toString()}; map keys=${[...map.keys()].join(',')}; update ${fullUpdate.length}B 写出`);
