# Модель глубокой подмены

## Основное правило

Цель — не максимальное число изменённых API, а единое правдоподобное
представление устройства во всех процессах и каналах. Значения образуют
`DeviceClass`: OS/architecture, Chromium major, display, graphics adapter,
locale/timezone, вычислительные и media capabilities. Случайность применяется
один раз при создании профиля и затем стабильна; независимый шум на каждый
вызов запрещён, поскольку ломает API и создаёт сильный отличительный признак.

Публичная схема отделяет:

- **declared values** — выбранные оператором locale, timezone, geolocation и
  proxy, целевые классы CPU/GPU и display;
- **derived values** — UA Client Hints, размеры viewport, media queries и
  codecs, вычисленные из класса;
- **stable seeds** — секретные per-profile seeds для допустимых детерминированных
  преобразований;
- **capability constraints** — то, что должна реально поддерживать сборка/ОС.

В UI нельзя независимо редактировать derived values. Изменение declared поля
пересчитывает кандидат и повторно валидирует весь профиль.

## Поверхности и единый источник

| Область | Наблюдаемые поверхности | Требование согласованности |
|---|---|---|
| Browser identity | User-Agent, UA-CH HTTP и JS, brands, platform, architecture, version | Только текущий Chromium major и поддерживаемая OS; HTTP и JS идентичны |
| Locale/time | `Accept-Language`, Navigator languages, ICU/Intl locale, timezone, date formatting | locale, timezone и геолокация согласуются с proxy-регионом либо конфликт явно подтверждён |
| Display/input | screen/work area, DPR, color depth/gamut, viewport, media queries, touch/pointer | размеры физически возможны; browser chrome и zoom учитываются единообразно |
| CPU/memory | UA-CH architecture/bitness, `hardwareConcurrency`, `deviceMemory`, WASM/SIMD и feature exposure, performance buckets | модель CPU задаётся классом; все доступные косвенные признаки соответствуют числу ядер, архитектуре и памяти |
| GPU | WebGL vendor/renderer/extensions/limits/shader precision/pixels, WebGPU adapter info/features/limits, Canvas/CSS rendering и codecs | vendor/model — часть GPU-класса; capabilities и rendering должны соответствовать ему, а не только заменённой строке |
| Audio | AudioContext properties и детерминированный rendered output | seed стабилен в профиле; значения остаются в допустимом диапазоне |
| Fonts/text | доступные fonts, enumeration, metrics, glyph rasterization | allowlisted font bundle соответствует OS/locale class и лицензиям |
| Media | devices, labels/IDs/group IDs, enumerate/getUserMedia, codecs | permission semantics сохранены; IDs origin-scoped и stable по правилам Chromium |
| Network/privacy | proxy, DNS route, WebRTC candidates, headers, IP exposure | весь origin traffic следует политике; fail closed при недоступном proxy |
| Device APIs | permissions, sensors, battery/gamepad where available | неподдерживаемое не выдумывается; одинаково в frames и workers |
| Storage/isolation | cookies, cache, storage quota, Service Workers | самостоятельный data directory и отсутствие cross-profile identifiers |

DOM prototypes, property descriptors, exception types, timing/order и Secure
Context/permission checks сохраняют стандартное поведение Chromium. Значение
не должно появляться через monkey patch там, где нативный Chromium его не
экспонирует.

## Уровни применения

1. **Browser/network process:** proxy, DNS, request headers, permissions,
   locale/timezone bootstrap и запуск процессов.
2. **Renderer/Blink/V8 bindings:** Navigator, Screen, Intl-facing configuration,
   media queries и workers из одного read-only snapshot.
3. **GPU/media:** реальные capability providers и output transformation там,
   где преобразование определено профилем.
4. **OS packaging:** fonts/resources и platform integration, необходимые
   конкретному поддерживаемому классу.

Предпочтителен выбор реально доступного backend и ограничение его capability
до согласованного подмножества. Подмена только строк WebGL/WebGPU запрещена.
Canvas/audio perturbation допустима лишь детерминированно, одинаково для
эквивалентного результата и без нарушения прозрачных пикселей, accessibility,
цветовых контрактов или пользовательского файла при экспорте.

## Подмена аппаратного профиля

Профиль обязательно содержит `HardwareClass` с architecture/bitness, классом
CPU, логическим числом ядер, bucket памяти, моделью GPU, graphics backend,
display и набором media capabilities. Панель позволяет выбрать класс из
versioned каталога и показывает все производные значения. Произвольная строка
«модель процессора» или «видеокарта» не принимается: она создала бы профиль,
которому противоречат feature tests и результаты рендеринга.

### CPU и память

Chromium adapter должен применять один hardware provider к UA-CH, Navigator,
WASM/V8 feature exposure и всем execution contexts. `hardwareConcurrency` и
`deviceMemory` подменяются нативно до создания renderer. Для выбранного класса
ограничивается доступный набор инструкций/возможностей, где Chromium позволяет
это сделать безопасно; недоступная хосту инструкция никогда не эмулируется
одной строкой. Производительность и timing нельзя точно превратить в другую
модель CPU, поэтому каталог объединяет модели в измеримо совместимые классы и
verification проверяет допустимые диапазоны, а не точную частоту.

### GPU

Выбор GPU управляет реальным graphics backend/provider. Он согласованно задаёт
WebGL/WebGPU adapter identity, extensions, limits, shader precision, codecs и
детерминированные результаты Canvas/WebGL. Предпочтительный порядок:

1. использовать реально доступный совместимый adapter;
2. ограничить capabilities до проверенного подмножества целевого GPU-класса;
3. использовать bundled software renderer для аппаратно-независимого класса,
   если его fingerprint целиком описан и протестирован;
4. отклонить профиль, если согласованный результат получить нельзя.

Замена только `UNMASKED_VENDOR_WEBGL`/`UNMASKED_RENDERER_WEBGL` либо WebGPU
adapter name запрещена. Для каждого GPU-класса golden tests проверяют строки,
extensions/limits, shader results, pixels и одинаковость данных в renderer и
GPU process.

## Региональная и сетевая идентичность

`RegionProfile` содержит страну, при необходимости административный регион и
город, IANA timezone, BCP 47 locale, упорядоченные languages, единицы/форматы и
опциональную geolocation с заданной точностью. Регион выбирается оператором и
не выводится молча из locale хоста.

По умолчанию locale, timezone, languages и geolocation вычисляются из выбранного
region preset. Оператор может выбрать другой валидный вариант региона. Любое
ручное несоответствие сохраняется как явный override с причиной; строгий режим
запрещает запуск с таким конфликтом.

Proxy является частью региона запуска, а не общей настройкой приложения. Для
профиля задаются HTTP(S) или SOCKS5 endpoint, ссылка на credentials, DNS mode и
fail-closed policy. До запуска preflight через тот же маршрут получает внешний
IP и region evidence. Несовпадение proxy-региона с `RegionProfile` блокирует
строгий запуск; при невозможности надёжно определить регион результат считается
`unverified`, а не автоматически успешным.

## IP, proxy, DNS и WebRTC

Proxy policy задаёт схему, endpoint, authentication reference, bypass list и
DNS mode. По умолчанию нет bypass кроме самого loopback control service.
Preflight проверяет внешний IP, DNS route и регион через диагностический
endpoint только с явного согласия оператора; отказ не раскрывает credentials.

UDP/WebRTC не может обходить выбранный маршрут. Если proxy не способен
переносить нужный UDP-трафик, профиль ограничивает candidates/transport либо
отклоняется — прямой fallback запрещён. Browser background traffic также
учитывается или отключается, если он не обязателен для продукта.

## Что не подменяется произвольно

TLS/HTTP2/HTTP3 fingerprint принадлежит конкретной сборке Chromium и её network
stack. Проект поддерживает только этот browser family/version и не заявляет
имитацию Safari или Firefox. Kernel scheduling, GPU driver, font rasterizer и
низкоуровневые timing side channels полностью скрыть внутри Chromium нельзя.
Automation/CDP, malware на host и внешние account/behavior signals также вне
модели. Эти границы должны показываться при выборе класса, а не скрываться за
оценкой «100% unique/safe».

## Эволюция схемы

Каждый профиль содержит `schema_version`, `generator_catalog_version`,
`chromium_compatibility` и provenance. Миграция чистая и однонаправленная,
создаёт backup и новый revision; downgrade не интерпретирует неизвестные поля.
Seed не меняется при миграции, если конкретное изменение явно не требует новой
identity и согласия оператора.
