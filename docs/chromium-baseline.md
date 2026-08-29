# Зафиксированная база Chromium

## Решение

Дата проверки: **2026-08-29 UTC**. Базовая линия первой реализации:

| Поле | Значение |
|---|---|
| Канал | Chrome for Testing Stable |
| Версия Chromium/Chrome | `152.0.7977.64` |
| Chromium revision | `1669021` |
| тег `chromium/src` | `152.0.7977.64` |
| commit `chromium/src` | `506c834ecceaa943c5f41e6cfe7f68acb5c45346` |

Версия получена из официального
[`last-known-good-versions-with-downloads.json`](https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json),
а соответствие тега commit проверено по официальному upstream через его
[GitHub mirror](https://github.com/chromium/chromium). Номер версии сам по себе
недостаточен: нормативным идентификатором исходников является полный commit.

`Chrome for Testing` используется только как официальный указатель стабильной
версии и источник эталонного поведения. Продукт собирается из Chromium source,
а не перепаковывает бинарный Chrome.

## Что именно фиксируется

До начала изменения Chromium создаётся машинно проверяемый baseline manifest:

- URL и полный commit `chromium/src`;
- commit `depot_tools`, которым выполнен `fetch`/`gclient sync`;
- сохранённый `DEPS` из этого commit и итоговые commits Git-зависимостей;
- instance ID CIPD-пакетов и digest всех загруженных архивов/toolchains;
- `gn args`, target platform/architecture, compiler и container/runner image;
- patch series проекта с порядком применения;
- SBOM, лицензии и SHA-256 выходных артефактов.

`gclient sync --revision src@506c...` получает дерево, но lock считается
полным только после сверки фактически разрешённых зависимостей с manifest.
Артефакты кэшируются в контролируемом immutable storage: upstream может
удалить исторический toolchain, даже если commit остался доступен. Крупное
дерево Chromium не включается в этот Git-репозиторий; здесь хранятся manifest,
patch series, build orchestration и собственный код.

## Ветки и обновления

- `baseline/152.0.7977.64` никогда не передвигается.
- Product changes основаны только на этой линии до принятия нового ADR.
- Security fix сначала переносится минимальным patch с upstream и проходит
  полный набор проверок. Если перенос опасен или невозможен, выпуск блокируется.
- Переход на следующую Stable создаёт новую baseline-ветку, заново разрешает
  зависимости и сравнивает поверхности fingerprint. Он не изменяет старые
  сборки и не мигрирует профиль молча.

Еженедельный мониторинг Chromium security releases создаёт issue с оценкой
применимости. Релиз с известной применимой критической уязвимостью запрещён,
даже если воспроизводимость старой линии идеальна.

## Приёмка фиксации

Два чистых изолированных runner должны получить одинаковые commits и собрать
функционально одинаковые пакеты. Где подпись, timestamp или platform packaging
делают побитовое совпадение невозможным, сравниваются неподписанная payload и
отдельно задокументированные nondeterministic поля.
