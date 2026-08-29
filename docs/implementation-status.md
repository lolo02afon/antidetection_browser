# Статус реализации Windows 11 privacy browser

Документ фиксирует фактический статус, а не расширяет требования. Целевое
назначение — безопасный серфинг, исследование fingerprinting и тестирование на
ресурсах, где оператор имеет право это делать. Обход антифрода и правил
сторонних сервисов не является обещанием продукта.

## Реализовано

- локальный control service и SPA с bearer-аутентификацией;
- типизированные профили, JSON-хранилище и optimistic revision;
- отдельный `user-data-dir`, runtime snapshot и проверка compatibility ID;
- профили ограничены Казахстаном (`kk-KZ`, `Asia/Almaty`/`Asia/Aqtobe`) и
  Бразилией (`pt-BR`, `America/Sao_Paulo`/`America/Manaus`);
- proxy endpoint принимается только как IPv4 с ненулевым портом; SOCKS5 требует
  DNS через proxy;
- proxy credentials намеренно не находятся в исходниках или профиле.

## Согласовано, но не реализовано

- загрузка зафиксированного Chromium baseline и воспроизводимая Windows 11
  toolchain/сборка;
- patch series Chromium и загрузка `ProfileConfig` до первого renderer;
- Windows Credential Manager и непрозрачный `credential_ref` для proxy;
- применение HTTP/SOCKS proxy в network process, запрет direct fallback,
  IPv6/QUIC/WebRTC/DNS leak policy и preflight внешнего IPv4;
- проверяемый каталог реальных CPU/GPU/display классов;
- согласованные browser/renderer/worker значения locale, timezone, UA-CH,
  display, media и graphics capabilities;
- consistency/leak/differential tests, параллельная изоляция и crash recovery;
- подписываемый Windows package, SBOM, provenance и clean-machine проверки.

До выполнения всех критериев `verification.md` результат является фундаментом
control plane, а не готовым browser distribution. Конкретные proxy credentials
не фиксируются ни в этом документе, ни в Git.
