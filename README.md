# Antidetection Browser

Репозиторий содержит документацию архитектуры и первый реализованный
вертикальный срез локального управления профилями. Control service уже хранит и
валидирует региональные/аппаратные профили, обслуживает локальную SPA и запускает
только совместимую модифицированную сборку Chromium через runtime snapshot.

## Запуск control service

```bash
cargo run --release -- \
  --chromium /path/to/patched/chromium \
  --data-dir ~/.local/share/antidetection-browser
```

Сервис слушает случайный порт только на `127.0.0.1` и печатает одноразовую URL с
bearer token во fragment. Обычный Chromium намеренно не запускается: executable
должен отвечать на `--antidetect-query-build-id` строкой
`antidetection/152.0.7977.64/profile-v1` и принимать
`--antidetect-profile-snapshot`. Это исключает тихий запуск без глубокой
подмены.

## Статус

Domain, JSON repository, optimistic revisions, Control API, SPA и безопасный
launcher contract реализованы. Патчи `chromium/src`, каталоги проверенных
регионов/аппаратных классов, proxy preflight и production packaging ещё не
реализованы; поэтому текущий commit не является готовым браузерным
дистрибутивом. Полный критерий готовности определён в
[`docs/verification.md`](docs/verification.md).
