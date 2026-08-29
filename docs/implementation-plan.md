# План реализации

План поставляет проверяемые вертикальные срезы. Следующий этап начинается после
приёмки обязательных доказательств предыдущего; наличие UI без Chromium patch не
считается готовым антидетект-браузером.

## 1. Control plane — реализован фундамент

- типизированные `RegionProfile`, `HardwareProfile` и `ProxyProfile`;
- полная validation перед сохранением и запуском;
- локальное JSON-хранилище с atomic replacement и optimistic revision;
- loopback Control API с bearer token и встроенная offline SPA;
- отдельный `user-data-dir`, private runtime snapshot и compatibility handshake;
- отказ от запуска обычного Chromium, который не подтверждает build ID.

Остаются до production-приёмки этапа: OS credential store, proxy credentials,
Host/Origin middleware, API integration tests, import/export и crash-safe locks.

## 2. Воспроизводимая база Chromium

1. Добавить baseline manifest с `chromium/src`, `depot_tools`, DEPS/CIPD и GN
   inputs, указанными в `chromium-baseline.md`.
2. Реализовать scripts resolve/sync/verify и immutable dependency cache.
3. Получить две чистые эталонные сборки и сравнить unsigned payload.
4. Ввести минимальную patch series и presubmit владельцев затронутого Chromium
   кода.

Результат: неизменяемое исходное дерево и воспроизводимая немодифицированная
сборка, поверх которой применяются только reviewable patches.

## 3. ProfileConfig в Chromium

1. Реализовать build-ID handshake и загрузку snapshot в browser process до
   создания NetworkContext/renderer.
2. Описать Mojo contracts для минимальных read-only частей browser, renderer,
   GPU и network process; snapshot digest возвращать launcher.
3. Добавить обязательную schema/build validation и fail-closed startup.
4. Покрыть main frame, iframe и worker contexts component tests.

Результат: профиль надёжно доставляется всем процессам, но ещё не заявляет
подмену конкретной поверхности.

## 4. Регион, IP и сеть

1. Добавить versioned каталог регионов: IANA timezone, BCP 47 locales,
   languages, formats и допустимая geolocation.
2. Реализовать proxy credentials через OS store, DNS mode, IPv4/IPv6 и
   per-profile NetworkContext.
3. Применить locale/timezone/languages/geolocation в Blink, ICU, headers и
   workers из одного provider.
4. Реализовать opt-in preflight внешнего IP/region и fail-closed WebRTC/QUIC/DNS
   leak policy.
5. Пройти controlled endpoint consistency/leak matrix.

## 5. Аппаратная идентичность

1. Создать измеренный каталог CPU/GPU/display классов из эталонных устройств.
2. Реализовать CPU provider: UA-CH architecture/bitness,
   `hardwareConcurrency`, `deviceMemory` и допустимый V8/WASM feature subset.
3. Реализовать GPU provider на границе GPU process: adapter identity, backend,
   extensions, limits, shader precision, codecs и rendering results.
4. Добавить поддерживаемый software-renderer class; невозможные сочетания
   отклонять, а не частично подменять.
5. Пройти golden/differential tests Canvas, WebGL, WebGPU, CSS и screenshots.

## 6. Остальные поверхности и изоляция

Последовательно, с отдельным provider и tests для каждого владельца: display и
input, fonts/text metrics, audio, media devices/permissions, storage quota и
фоновые network features. Затем — параллельные профили, migration, recovery и
удаление без cross-profile данных.

## 7. Выпуск

Собрать platform packages, SBOM/provenance/licenses, signing/notarization,
security triage и clean-machine install/update/uninstall. Выпуск получает статус
готового продукта только после всех критериев `verification.md`.

## Порядок ближайших изменений

Следующий pull request должен завершить production-приёмку control plane, затем
идут baseline tooling и сквозной `ProfileConfig` patch. Подмена отдельных API до
этого запрещена: она создала бы частично применяемый и обнаружимый профиль.
