# Архитектура

## Контексты и направление зависимостей

```text
Profile UI (SPA) -> Local Control API -> Application use cases
                                         |              |
                                  Profile domain   Browser launcher
                                         |              |
                                  encrypted store  Chromium adapter
                                                        |
                                browser process -> renderer/GPU/network
```

Внутреннее правило зависимости направлено к domain: профиль и его инварианты
не знают о JSON, HTTP, SQLite, Chromium flags или Mojo. Application layer
реализует сценарии и зависит от малых интерфейсов (`ProfileRepository`,
`SecretStore`, `BrowserRuntime`, `Clock`). Infrastructure adapters реализуют
их. UI использует версионированный Control API и не читает хранилище напрямую.
Это применяет SOLID без искусственной иерархии: одна причина изменения у
компонента, расширение через adapters, substitutable contracts, узкие
интерфейсы и dependency inversion.

Наследование применяется только при реальном отношении substitutability;
варианты поверхности профиля собираются композиционно. Единственным владельцем
нормализованного состояния является immutable `FingerprintProfile` value
object. Chromium-specific DTO не проникают в domain.

## Компоненты

### Profile domain

Определяет схему, типы, ranges, cross-field invariants, региональные и
аппаратные классы, версию формата и
операции создания/изменения. Отдельный `ProfileValidator` возвращает все
нарушения с путями полей. Генератор выбирает только из версионированного
каталога поддерживаемых согласованных классов; он не генерирует независимо
случайные поля.

### Local control service

Единственный процесс-владелец профилей, секретов и запусков. Он обслуживает SPA
только на loopback, сериализует изменение одного профиля, ведёт audit событий
без значений credentials и удерживает lock каталога данных. API имеет
optimistic revision для защиты от потерянных обновлений.

### Browser launcher

Создаёт отдельный `user-data-dir`, проверяет отсутствие активного экземпляра,
разрешает secrets proxy, формирует подписанный runtime snapshot, запускает
browser process и ждёт readiness. Snapshot содержит schema version, profile
revision, build compatibility ID и nonce; после запуска он неизменяем.

### Chromium integration

Минимальный patch добавляет типизированный `ProfileConfig` в browser process и
явные adapters у владельцев наблюдаемого поведения Chromium. Browser process
передаёт renderer/GPU/network только нужные read-only части через существующие
IPC/Mojo boundaries. Запрещены scattered command-line switches, JavaScript
injection, extension overrides и изменение built-ins после загрузки страницы:
они запаздывают, неполны и сами обнаружимы.

Каждая поверхность имеет одного владельца-провайдера. Например, Blink получает
navigator/display/Intl значения через platform interfaces, network service —
headers/proxy/DNS, content permissions — media/permission identities, GPU
process — согласованные GPU identity, backend и capabilities, V8/renderer —
CPU architecture/features, cores и memory bucket. И обычный API, и
Worker/iframe/worklet используют один snapshot.

### Storage

Метаданные и несекретные значения хранятся транзакционно локально. Proxy
passwords и ключ шифрования — в OS credential store; экспорт по умолчанию их
не содержит. Каждый профиль имеет самостоятельный Chromium data directory.
Удаление сначала останавливает runtime, затем атомарно исключает профиль из
реестра и удаляет его данные с понятной диагностикой частичного filesystem
сбоя.

## Жизненный цикл запуска

1. Control service загружает профиль и проверяет schema/build compatibility.
2. Validator проверяет поля, согласованность, host capabilities и proxy/DNS.
3. Launcher получает exclusive lock и материализует runtime snapshot.
4. Chromium принимает snapshot до инициализации network context и renderer.
5. Browser возвращает применённые profile ID/revision/digest; несовпадение
   останавливает процесс.
6. После exit service фиксирует причину, освобождает lock; snapshot уничтожается.

Изменение сохранённого профиля не меняет уже запущенную сессию. Для применения
нужен явный restart — это устраняет состояние, где разные процессы видят
разные версии.

## Наблюдаемость и ошибки

Логи связываются `launch_id`, `profile_id` и build ID, но не содержат proxy
credentials, browsing data и точных стабильных идентификаторов устройства.
Ошибки разделены на validation, compatibility, storage, proxy и runtime; UI не
заменяет их общим «не удалось». Crash reports выключены по умолчанию и могут
отправляться только после информированного согласия с redaction.
