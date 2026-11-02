# Building the documentation / Сборка документации

```powershell
python reference/build-site.py .
```

Run at the root of this documentation repository with Python 3. The builder uses the local Markdown, translated labels, CSS, JavaScript and captured measurement data. It does not run benchmarks, download dependencies or execute the native code generator. Both index.html and the standalone RU/EN page are regenerated.

Исходники интерфейса находятся в docs/assets: full.js отвечает за измерения, explorers.js — за конкретные интерактивные модели, explorers.html — за их разметку. Тексты RU/EN находятся в docs/data/full-words.json. Измерения и модели имеют разные наборы данных: модели не добавляют выдуманных временных оценок.

For the native byte-model reference, see [docs-lab](docs-lab/README.md). Browser visual QA is not claimed; offline DOM and native-vector checks are recorded in [the audit](../SOURCE_AUDIT.md).
