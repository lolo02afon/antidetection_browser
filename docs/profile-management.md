# Управление профилями

## Решение по форме панели

Панель — локальное web-приложение (SPA). Её статические assets входят в пакет
продукта, обслуживаются локальным control service и открываются на случайном
loopback-порту. Это сохраняет разделение UI/domain, не встраивает настройки в
Chromium UI и не требует отдельного desktop framework.

Cloudflare Pages, GitHub Pages и другое внешнее размещение **не используются**.
Панель должна работать без Интернета и всегда соответствовать версии локальных
API и схемы профиля. Внешнему сайту пришлось бы обращаться к localhost, что
добавило бы DNS rebinding/CORS риски, зависимость от доступности третьей стороны
и новый supply-chain канал без пользы для обязательных сценариев.

## Control API

API версионируется (`/api/v1`) и предоставляет только сценарии:

- список/получение/создание/клонирование/изменение/удаление профиля;
- validate и preview вычисленных значений;
- выбор региона, proxy и согласованного аппаратного класса;
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

Редактор отдельно показывает регион и сеть, software identity и hardware
identity. Оператор выбирает регион, proxy и поддерживаемые классы CPU/GPU;
locale/timezone/geolocation и производные аппаратные значения показываются до
сохранения. Ручное отклонение locale/timezone от региона возможно только в
advanced mode с явным конфликтом, который остаётся виден при каждом запуске.

Редактор группирует исходные declared values, показывает derived read-only и
объясняет cross-field conflict у конкретных полей. Перед запуском видны proxy
policy, фактически проверенный внешний IP и его регион, build/profile revision
и несовместимости. Состояния
`stopped`, `starting`, `running`, `stopping`, `failed` показываются явно;
кнопки недоступных переходов отсутствуют.

Удаление требует подтверждения, работающий профиль сначала корректно
останавливается. Export явно сообщает об исключённых secrets. Import сначала
показывает migration/compatibility report и ничего не меняет до подтверждения.
