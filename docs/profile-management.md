# Управление профилями

## Решение по форме панели

Панель — web-приложение (SPA), но production-копия поставляется как статические
assets внутри локального control service и открывается на случайном loopback
порту. Это сохраняет разделение UI/domain, не встраивает настройки в Chromium
UI и не требует отдельного desktop framework. Внешний сайт не может быть
единственным production UI: ему пришлось бы обращаться к localhost, что
создаёт риски DNS rebinding/CORS, зависимости от сети и несовпадения версий.

GitHub Pages подходит только для публичной документации и UI preview. Оно
публикует статические файлы и не является приватным control plane.
Cloudflare Pages также допустим для preview; Functions/Workers превращают
решение в удалённый backend с отдельной threat model, а бесплатные лимиты и
условия могут меняться. Ни один публичный deployment не получает доступ к
локальным профилям или секретам. Production assets собираются из того же
commit и сверяются по digest.

## Control API

API версионируется (`/api/v1`) и предоставляет только сценарии:

- список/получение/создание/клонирование/изменение/удаление профиля;
- validate и preview вычисленных значений;
- start/stop/status и поток ограниченных runtime events;
- export/import без секретов по умолчанию;
- сведения о build, schema compatibility и ожидающем обновлении.

UI не может передать произвольные Chromium flags, пути к executable или
невалидированный JSON. Mutations требуют текущую profile revision и
idempotency key. OpenAPI contract служит границей и генерирует клиентские
типы, но domain validation остаётся на сервере.

## Локальная безопасность

- service слушает только `127.0.0.1`/`::1`, никогда wildcard interface;
- при старте создаёт high-entropy session token и передаёт его UI через
  fragment/одноразовый bootstrap, не query и не persistent storage;
- проверяет `Host`, `Origin`, method и content type; CORS для внешних origins
  отсутствует, DNS rebinding блокируется;
- state-changing запросы требуют token и CSRF proof; cookies, если появятся,
  имеют `HttpOnly`, `SameSite=Strict` и ограниченный lifetime;
- CSP запрещает remote scripts, frames и произвольные connections; assets не
  загружают CDN analytics;
- открытие браузера выполняется через OS API без shell interpolation;
- control service проверяет права каталога/IPC и не доверяет UI validation.

TLS на loopback без доверенного локального сертификата не добавляет защиты;
граница обеспечивается bind/auth/origin controls. Если позже появится remote
management, оно проектируется как отдельный продукт с TLS, identity, RBAC,
revocation и end-to-end threat model, а не включается флагом.

## UX обязательных сценариев

Редактор группирует исходные declared values, показывает derived read-only и
объясняет cross-field conflict у конкретных полей. Перед запуском видны proxy
policy, ожидаемый регион, build/profile revision и несовместимости. Состояния
`stopped`, `starting`, `running`, `stopping`, `failed` показываются явно;
кнопки недоступных переходов отсутствуют.

Удаление требует подтверждения, работающий профиль сначала корректно
останавливается. Export явно сообщает об исключённых secrets. Import сначала
показывает migration/compatibility report и ничего не меняет до подтверждения.
