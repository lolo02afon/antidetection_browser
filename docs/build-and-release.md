# Сборка и выпуск

## Репозиторий и patch discipline

Собственный репозиторий содержит:

```text
build/        получение baseline, manifests, GN presets, packaging
patches/      упорядоченные минимальные patches к chromium/src
src/          control service, domain и UI
schemas/      profile и Control API contracts
tests/        unit, integration, conformance и system fixtures
docs/         нормативная архитектура и решения
```

Каждый Chromium patch имеет одну ответственность, upstream base commit и тест.
Generated patch series применяется к чистому baseline без ручных действий.
Изменения upstream-кода держатся у существующего владельца поведения Chromium;
не создаётся параллельный fork Blink API.

## Конвейер

1. **Resolve:** сверить baseline manifest, commits, CIPD IDs и digest.
2. **Patch:** применить series с запретом fuzz/reject.
3. **Generate:** выполнить `gn gen` с versioned release args.
4. **Build:** собрать Chromium, control service и локальную SPA в чистом runner.
5. **Test:** unit/component, Chromium targeted tests, integration, fingerprint
   consistency, proxy leak, profile isolation и smoke на каждой ОС.
6. **Package:** включить runtime assets, licenses, default policy и build info;
   исключить test keys и developer endpoints.
7. **Attest:** создать SBOM/provenance, hashes и vulnerability report.
8. **Sign/notarize:** platform signing отделено от воспроизводимой payload.
9. **Promote:** один проверенный digest перемещается между каналами, не
   пересобирается.

Release build запрещает remote debugging по умолчанию, test-only switches,
неприкреплённые endpoints и updater, способный взять официальный Chrome вместо
product build. Обновление скачивает только подписанный product manifest,
проверяет совместимость профиля и предоставляет rollback пакета без rollback
уже мигрировавшего хранилища.

## Платформы

Матрица отдельно фиксирует `windows-x64`, `macos-arm64/x64` и `linux-x64`.
Один profile class не объявляется переносимым между OS, пока conformance не
докажет это. Подпись Windows, notarization macOS и package formats Linux —
отдельные adapters поверх одинаковой неподписанной payload model.

## Готовность выпуска

Пакет готов только при зелёных обязательных проверках, отсутствии применимой
необработанной critical vulnerability, наличии подписей/SBOM/licenses и
прохождении clean-machine install/start/update/uninstall. Ручное наблюдение
fingerprint-сайта полезно для исследования, но не заменяет versioned assertions.
