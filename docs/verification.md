# Стратегия проверки

## Уровни доказательства

### Domain

Property-based tests генерируют допустимые и недопустимые сочетания и проверяют
инварианты, стабильную сериализацию и миграции. Golden fixtures представляют
каждый поддерживаемый `DeviceClass`, но не заменяют проверки свойств.

### Chromium component

Targeted browser/Blink/network/GPU tests доказывают, что каждый provider читает
snapshot и сохраняет стандартные descriptors, permissions и errors. Отдельно
проверяются main frame, same/cross-origin iframe, dedicated/shared/service
workers, worklets и incognito, если он поддерживается.

### Сквозная consistency matrix

Локальная test page собирает значения из JS, request headers и server-observed
network. Assertions сравнивают не только expected значения, но и отношения:

- UA ↔ UA-CH ↔ Chromium build ↔ OS/architecture;
- locale ↔ languages ↔ `Accept-Language` ↔ Intl;
- timezone ↔ geolocation ↔ явно заданная proxy policy;
- выбранный регион ↔ фактический proxy IP/region ↔ locale/timezone/languages;
- screen ↔ DPR ↔ viewport ↔ media queries ↔ screenshots;
- CPU class ↔ UA-CH architecture/bitness ↔ cores/memory ↔ WASM/V8 features;
- GPU model ↔ WebGL/WebGPU identity ↔ extensions/limits ↔ Canvas/shaders/codecs;
- media device IDs/permissions между reload, origins и профилями;
- одинаковый snapshot во всех execution contexts;
- стабильность в одном профиле и предусмотренное различие двух профилей.

### Leak и изоляция

Контролируемые proxy/DNS/WebRTC endpoints проверяют отсутствие прямого traffic
при нормальной работе, отказе proxy, auth failure, IPv4/IPv6 и QUIC/UDP.
Filesystem/network instrumentation проверяет, что разные профили не используют
общие data directories и identifiers. Секреты не появляются в logs, crash
dumps, export или command line.

### Регрессия и внешнее наблюдение

Сохраняются versioned fingerprint snapshots и различия объясняются в review.
Несколько независимых публичных test pages применяются как ненормативные
smoke checks: их оценки меняются без контроля проекта и не являются критерием
«невидимости». Более сильное доказательство — собственные assertions плюс
differential run эталонного Chromium той же версии на реальном устройстве
поддерживаемого класса.

## Нефункциональные проверки

- clean builds дважды для reproducibility;
- Chromium security/unit test subsets затронутых owners;
- API schema compatibility и malformed/untrusted import fuzzing;
- crash recovery, concurrent start, disk-full и interrupted migration;
- CSP/dependency audit панели, loopback auth, Origin/Host и DNS rebinding tests;
- cold-start и page-load overhead относительно baseline с заранее утверждённым
  бюджетом перед реализацией каждой дорогой трансформации;
- accessibility и platform install/update/uninstall.

## Критерии готовности продукта

1. Baseline воспроизводимо разрешается по manifest и полный patch series
   применяется к чистому tree.
2. Все обязательные поверхности поддерживаемого класса проходят consistency
   matrix; неизвестная/непроверенная поверхность не объявляется подменённой.
3. Proxy failure закрывает network, leak tests проходят на IPv4/IPv6/WebRTC.
4. Два профиля одновременно изолированы; restart сохраняет identity профиля.
5. Любой invalid/incompatible profile отклоняется до первого renderer.
6. Панель выполняет полный жизненный цикл без доступа внешнего origin к API.
7. Подписанный пакет проходит clean-machine сценарий, содержит provenance,
   SBOM, licenses и не имеет необработанной применимой critical vulnerability.

Если хотя бы один пункт не доказан, результат называется исследовательской
сборкой, а не готовым антидетект-браузером.
