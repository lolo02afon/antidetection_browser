# Первичные источники

Источники задают проверяемую основу решений; детали проекта определяются
остальными документами, а не копируются отсюда.

## Chromium и воспроизводимая база

- [Chrome for Testing: version selection and JSON endpoints](https://github.com/GoogleChromeLabs/chrome-for-testing#json-api-endpoints) — официальный машиночитаемый указатель каналов и версий.
- [Chromium source mirror](https://github.com/chromium/chromium) — зеркало official source и разрешение зафиксированного тега в commit.
- [Checking out and building Chromium](https://chromium.googlesource.com/chromium/src/+/main/docs/get_the_code.md) — `depot_tools`, `fetch`, `gclient` и platform instructions.
- [Chromium DEPS format](https://chromium.googlesource.com/chromium/src/+/main/docs/dependencies.md) — модель внешних зависимостей.
- [GN build configuration](https://chromium.googlesource.com/chromium/src/+/main/tools/gn/docs/quick_start.md) — generation и build arguments.
- [Chromium security release process](https://chromium.googlesource.com/chromium/src/+/main/docs/security/faq.md) — причина не считать pin отказом от security maintenance.

## Границы процессов и web platform

- [Chromium multi-process architecture](https://www.chromium.org/developers/design-documents/multi-process-architecture/) — browser/renderer separation.
- [Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) — сохранение privilege boundaries.
- [Mojo documentation](https://chromium.googlesource.com/chromium/src/+/main/mojo/README.md) — типизированные IPC contracts.
- [User-Agent Client Hints](https://wicg.github.io/ua-client-hints/) — согласованность HTTP и Navigator surfaces.
- [WebRTC IP address handling](https://www.w3.org/TR/webrtc/#revealing-ip-addresses) — privacy requirements для ICE candidates.
- [WebGPU](https://www.w3.org/TR/webgpu/) и [WebGL](https://registry.khronos.org/webgl/specs/latest/1.0/) — graphics capabilities и observable contracts.
- [Permissions](https://www.w3.org/TR/permissions/) и [Media Capture and Streams](https://www.w3.org/TR/mediacapture-streams/) — permission/device semantics.

## Локальная панель

- [OWASP DNS rebinding prevention](https://cheatsheetseries.owasp.org/cheatsheets/DNS_Rebinding_Attack_Prevention_Cheat_Sheet.html) — защита loopback API.
- [W3C Content Security Policy](https://www.w3.org/TR/CSP3/) — ограничение источников локальной SPA.

## Ограничение актуальности

Версия Chromium меняется. Точная версия принадлежит `chromium-baseline.md`;
production control plane остаётся локальным и не зависит от внешнего hosting.
