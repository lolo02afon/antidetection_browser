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
  proxy;
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
| Compute/memory | `hardwareConcurrency`, `deviceMemory`, WASM/feature exposure | bucket не превышает совместимый класс host/runtime |
| Graphics | Canvas 2D, WebGL strings/capabilities/pixels, WebGPU adapter/features, CSS rendering | один graphics class; нельзя менять только vendor/renderer, оставив противоречивые limits |
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

## IP, proxy, DNS и WebRTC

Proxy policy задаёт схему, endpoint, authentication reference, bypass list и
DNS mode. По умолчанию нет bypass кроме самого loopback control service.
Preflight проверяет внешний IP и DNS через диагностический endpoint только с
явного согласия оператора; отказ не раскрывает credentials.

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
