# Phase Request Template

Use this prompt when asking an agent to work on a feature.

```markdown
対象仕様:
- docs/specs/<feature>.md

現在のフェーズ:
- Phase <1|2|3>: <Red|Green|Refactor>

制約:
- フェーズを飛び越えないこと。
- Feature Path として最初に [THOUGHT] を出力すること。
- 外部リソースは必ずPort/Interface経由にすること。
- 仕様にない挙動を推測して追加しないこと。

依頼:
<具体的な依頼>
```
