# Захват одного монитора, выхода и микрофона на Hyprland

## Вопрос

Как на Hyprland (CachyOS, PipeWire) отдельная программа может с входа в сессию держать кольцо одного монитора, звука выхода и микрофона — и что из этого реально выдерживает кольцо длиной 10–20 минут?

Ограничения, которые уже решены и здесь не пересматриваются: отдельное фоновое приложение, автозапуск вместе с сессией Hyprland, жизнь отдельно от оболочки; кольцо — один выбранный монитор (задуманный умолчанием «primary») плюс системный выход и микрофон; длина 15 секунд – 20 минут, по умолчанию 10 минут, 60 кадров/с; запись уже идёт после входа, без открытия окна приложения. NVIDIA ShadowPlay как механизм недоступен. Конкретный производитель GPU не предполагается.

Источники ниже — документация и код самих проектов. Отдельного первичного источника по патчам CachyOS нет: механизм берётся из Hyprland, PipeWire, WirePlumber и xdg-desktop-portal.

## Короткий ответ

Обычный пользовательский процесс может снимать один выход двумя путями. Прямой Wayland-протокол (`zwlr_screencopy_manager_v1`, а с Hyprland 0.54.0 ещё и `ext-image-copy-capture-v1`) копирует следующий кадр выбранного `wl_output` в буфер клиента и диалога портала не показывает. Портал `org.freedesktop.portal.ScreenCast` отдаёт один PipeWire-поток монитора, но первый запуск показывает выбор, а тихий повтор возможен только после сохранённого одноразового restore token.

Звук выхода и микрофон — два независимых потока захвата PipeWire в той же сессии: мониторные порты текущего sink по умолчанию и source по умолчанию. Отдельного диалога на это в документации PipeWire нет.

Сырые кадры 60 fps на 10–20 минут в память не помещаются: 1080p в 32-битном кадре — это порядка 280 ГиБ за 10 минут и 560 ГиБ за 20. Протоколы отдают несжатый кадр и кольца не хранят. Кольцо такой длины в первичных источниках показано как закодированный буфер (gpu-screen-recorder держит replay в RAM именно как сжатое видео и прямо предупреждает не раздувать длительность). Гарантии «ровно 60 уникальных кадров» и «монитор primary» протоколы не дают.

## Находки

### 1. Один монитор потоком из обычного процесса

Есть два поддержанных механизма. Оба доступны процессу пользователя, у которого есть сокет Wayland этой сессии. Ни один из них не является непрерывным закодированным потоком: клиент получает кадры.

**Прямое копирование кадра выхода.**

`zwlr_screencopy_manager_v1.capture_output` копирует следующий кадр целого `wl_output` в буфер клиента. Композитор перечисляет параметры `wl_shm` и, если умеет, `linux-dmabuf`, затем клиент шлёт `copy` или `copy_with_damage` и ждёт `ready` или `failed`. На один объект кадра приходится одно копирование; для серии кадров клиент запрашивает кадр снова. Протокол сам помечен как устаревший и отсылает к `ext-image-copy-capture-v1`. Таблица поддержки на Wayland Explorer относит к Hyprland 0.52.1 версию 3 этого протокола.

Источник: [wlr-screencopy-unstable-v1](https://wayland.app/protocols/wlr-screencopy-unstable-v1).

Сменяющий его протокол `ext-image-copy-capture-v1` в статусе testing. Сессия создаётся из источника, композитор присылает `buffer_size`, форматы SHM и DMA-BUF и событие `done`. Одновременно у сессии может быть только один объект кадра. Источник выхода создаёт `ext_output_image_capture_source_manager_v1.create_source(output)`: картинка совпадает с выходом, курсор и помеченные прозрачными оверлеи могут отсутствовать.

Источники: [ext-image-copy-capture-v1](https://wayland.app/protocols/ext-image-copy-capture-v1), [ext-image-capture-source-v1](https://wayland.app/protocols/ext-image-capture-source-v1).

Hyprland 0.54.0 включает реализацию обоих протоколов (`image-capture-source-v1` и `image-copy-capture-v1`) и при этом оставляет `zwlr_screencopy_manager_v1`. Патч 0.54.3 чинит обработку stop у image-copy-capture, то есть протокол уже в ветке 0.54, а не только в невыпущенном дереве. Строка Wayland Explorer про Hyprland 0.52.1 этот протокол ещё не показывает: на 0.52 его нет.

Источники: [релиз v0.54.0](https://github.com/hyprwm/Hyprland/releases/tag/v0.54.0) (пункт «protocols: implement image-capture-source-v1 and image-copy-capture-v1»), [PR #11709](https://github.com/hyprwm/Hyprland/pull/11709), [релиз v0.54.3](https://github.com/hyprwm/Hyprland/releases/tag/v0.54.3).

Вики Hyprland относит прямой захват без портала к инструментам grim, wl-screenrec и wf-recorder и называет это правом `screencopy`. Та же вики рекомендует для записи wf-recorder и OBS. OBS на Hyprland идёт через PipeWire и портал.

Источники: [Permissions](https://wiki.hypr.land/Configuring/Advanced-and-Cool/Permissions/), [Screenshots & Recording](https://wiki.hypr.land/Useful-Utilities/Screenshots-and-Recording/).

**Портал ScreenCast, один монитор, поток PipeWire.**

Интерфейс `org.freedesktop.portal.ScreenCast` (документация версии 6): `CreateSession`, один раз `SelectSources`, затем `Start`. В ответе `Start` — массив потоков PipeWire. Если `multiple` ложно, массив содержит один поток. Тип источника `1` — MONITOR, существующие мониторы. `OpenPipeWireRemote` отдаёт fd, из которого клиент делает `pw_context_connect_fd`; на этом соединении видны только узлы этой сессии захвата. С версии 6 для привязки к узлу указан `pipewire-serial` и `PW_KEY_TARGET_OBJECT`, потому что числовой node id PipeWire переиспользуется.

Источник: [ScreenCast](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html).

xdg-desktop-portal-hyprland — реализация портала Hyprland. Её поднимает D-Bus после старта Hyprland. Вики описывает проверку так: шаринг или OBS с источником PipeWire открывает Qt-меню выбора. В `~/.config/hypr/xdph.conf` секция `screencopy` задаёт `max_fps` (по умолчанию 120, `0` — без предела) и `force_shm` (пропустить DMA-BUF и всегда копировать через SHM; SHM медленнее, особенно на высоком разрешении). Окно шаринга на других wlroots-композиторах через XDPH не работает, захват выхода работает.

Источник: [xdg-desktop-portal-hyprland](https://wiki.hypr.land/Hypr-Ecosystem/xdg-desktop-portal-hyprland/).

В коде XDPH восстановленная сессия типа output хранит имя выхода. Если данные восстановления годны, портал пишет в лог, что диалог не показывается. Для окна токен временный: закрытое окно, перезапуск и перезагрузка его не переживают. Это слова сопровождающего в тикете, не гарантия спецификации портала.

Источники: сравнение тегов в [v1.3.11...v1.3.12](https://github.com/hyprwm/xdg-desktop-portal-hyprland/compare/v1.3.11...v1.3.12) (`TYPE_OUTPUT` кладёт `output` в restore struct; ветка «restore data valid, not prompting»), [issue #350](https://github.com/hyprwm/xdg-desktop-portal-hyprland/issues/350).

**Что такое «один монитор» и почему это не «primary».**

Оба прямых протокола принимают конкретный `wl_output`. Имя выхода в конфиге Hyprland — строка коннектора, пример `monitor = DP-1, 1920x1080@144, 0x0, 1`. Список выходов — `hyprctl monitors` / `hyprctl monitors all`. Событие IPC `focusedmon` сообщает имя активного монитора. Флага «primary» у Wayland-выхода вики не описывает. `xrandr --output … --primary` в FAQ относится к тому, какой монитор X11-приложения считают основным.

Источники: [Monitors](https://wiki.hypr.land/0.53.0/Configuring/Monitors/), [IPC](https://wiki.hypr.land/IPC/), [FAQ, абзац про разрешение игр на втором мониторе](https://wiki.hypr.land/FAQ/).

Портал не принимает имя коннектора аргументом. Монитор выбирает пользователь в диалоге либо восстановление по ранее сохранённому токену. Спецификация прямо говорит: если сохранённый монитор больше недоступен, токен игнорируется и пользователя спрашивают снова.

**60 кадров в секунду протоколом не обещаны.** `capture_output` копирует следующий кадр выхода. Клиент сам задаёт темп запросов. `max_fps` портала Hyprland ограничивает сессию шаринга сверху (по умолчанию 120). Частота самого выхода задаётся режимом монитора (`1920x1080@144` в примере вики) и может быть и ниже, и выше 60. Спека может требовать от приложения выборку 60 Гц. Утверждать, что композитор отдаст 60 различных кадров в секунду, нельзя.

**Автозапуск без окна и отдельно от оболочки.**

С Hyprland 0.55 автозапуск — обработчик `hyprland.start` и `hl.exec_cmd()`. Команда стартует один раз при запуске композитора, асинхронно. На 0.54 то же самое делает `exec-once`: только при запуске, не при каждой перезагрузке конфига. FAQ приводит тот же `hyprland.start` как способ открыть приложения при входе.

Источники: [Autostart](https://wiki.hypr.land/Configuring/Basics/Autostart/), [Keywords, старая страница](https://github.com/hyprwm/hyprland-wiki/blob/5e1782bd/content/Configuring/Keywords.md), [FAQ](https://wiki.hypr.land/FAQ/).

Сессия через uwsm считает штатным путём systemd user unit или XDG autostart, а не дочерний процесс юнита композитора. XDG autostart в такой сессии включает systemd. Юнит можно повесить на `graphical-session.target`.

Источник: [Systemd startup](https://wiki.hypr.land/Useful-Utilities/Systemd-start/).

Захват привязан к сокету Wayland этого композитора. Когда Hyprland завершается, этот сокет исчезает, и копирование кадров продолжаться не может. Вики не пишет, убивает ли композитор сам процесс `exec-once` при выходе. Перезагрузка конфига `exec-once` заново не запускает: уже запущенный процесс от этого события не зависит. Перезапуск композитора начинает новую сессию захвата. Кольцо в памяти прошлой сессии протоколы не переносят.

gpu-screen-recorder показывает тот же рисунок автозапуска: user-юнит `gpu-screen-recorder.service` с `WantedBy=default.target`, `Restart=on-failure`, без GUI. В поставленном юните окно захвата — `screen`, частота 60, replay 60 секунд, звук `default_output`, второй `-a` пустой. Это доказательство, что replay умеет стартовать вместе с пользовательской сессией. Это не доказательство, что прямой захват монитора у этой программы обходится без прав root: её README пишет, что запись монитора ходит в KMS и просит root, а без особых прав на Wayland остаётся `-w portal` (или окно). Для нашего процесса этот KMS-путь не подходит.

Источники: [README gpu-screen-recorder](https://git.dec05eba.com/gpu-screen-recorder/plain/README.md) (разделы Replay, AMD/Intel/Wayland root permission, Run replay on system startup), [extra/gpu-screen-recorder.service](https://git.dec05eba.com/gpu-screen-recorder/plain/extra/gpu-screen-recorder.service).

### 2. Диалог: каждый вход, один раз или никогда

Зависит от пути.

**Прямой протокол, принуждение прав выключено.** Система прав Hyprland выключена, пока её явно не включили (`ecosystem.enforce_permissions` в старом синтаксисе, `hl.config({ ecosystem = { enforce_permissions = true } })` в текущем). В этом состоянии вики не описывает ни диалог портала, ни popup прав. Захват одного `wl_output` с входа в сессию может идти без вопроса пользователю. Это единственный путь, для которого «никогда» задокументировано как поведение по умолчанию.

**Прямой протокол, принуждение включено.** Право называется `screencopy`, значение по умолчанию — `ask`. Popup предлагает отказать, разрешить до выхода приложения или разрешить до выхода Hyprland. Режим `allow` в конфиге разрешает бинарю по regex пути без вопроса. Правила из конфига подхватываются только после рестарта Hyprland. Сам xdg-desktop-portal-hyprland — обычный клиент и тоже проходит через это право; вики даёт пример `allow` для его бинаря. Отказ рисует чёрный кадр с текстом permission denied.

PR, который добавил image-copy-capture, проверял то же поведение прав: чёрный кадр, пока пользователь не разрешил или не отказал, для portal+OBS, wl-screenrec и grim. Вики при этом называет ключ конфига `screencopy` и примеры grim / wl-screenrec / wf-recorder. Отдельной строки «ключ `screencopy` покрывает ext-image-copy-capture» в вики нет; совпадение следует из описания права («доступ к экрану мимо портала») и из теста PR.

Источник: [Permissions](https://wiki.hypr.land/Configuring/Advanced-and-Cool/Permissions/), [PR #11709](https://github.com/hyprwm/Hyprland/pull/11709).

Как часто `ask` срабатывает внутри уже идущего захвата, вики не уточняет. Написано «каждый раз, когда приложение пытается сделать чувствительное действие». Для спеки безопасно сказать: без правила `allow` при включённых правах вопрос будет. Небезопасно обещать, что вопрос ровно один на процесс или ровно один на кадр.

**Портал.** `Start()` по спецификации обычно показывает диалог выбора. `persist_mode`: `0` — не хранить, `1` — пока приложение живо, `2` — пока пользователь явно не отзовёт. Токен одноразовый: следующий старт возвращает новый, и приложение обязано сохранить его само. Если восстановление не удалось, пользователя спрашивают как обычно.

Источник: [ScreenCast, Session persistence и SelectSources](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html).

У XDPH галочка «разрешить restore token» по умолчанию выключена. `screencopy:allow_token_by_default` только заранее отмечает её. Первый запуск всё равно проходит через меню. Релиз 1.3.5 чинил потерю restore data после двойного рестарта приложения.

Источники: [вики XDPH, `allow_token_by_default`](https://wiki.hypr.land/Hypr-Ecosystem/xdg-desktop-portal-hyprland/), [релиз v1.3.5](https://github.com/hyprwm/xdg-desktop-portal-hyprland/releases/tag/v1.3.5).

Постоянные права лежат в permission store и ключуются `app_id`. Пустой `app_id` с портала 1.20 — общий только для тех приложений, чей id определить не удалось, а не для всех несэндбоксных программ. Хостовое приложение может до любого вызова портала вызвать `org.freedesktop.host.portal.Registry.Register`; id должен совпадать с именем `.desktop`-файла. Регистрация временная, интерфейс могут убрать. Вики Hyprland не пишет, кладёт ли `hl.exec_cmd` процесс в cgroup с app id.

Источники: [xdp-session-persistence.c](https://github.com/flatpak/xdg-desktop-portal/blob/main/desktop-portal/xdp-session-persistence.c), [NEWS 1.20, empty app IDs](https://github.com/flatpak/xdg-desktop-portal/blob/1.20.0/NEWS.md), [Registry](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html).

Итог по порталу: первый успешный вход с диалогом и согласием на токен; следующие входы без диалога, если токен сохранён приложением, `app_id` тот же, выход с тем же именем ещё есть, а реализация портала смогла восстановить сессию. Иначе диалог снова. «Никогда» для портала спецификация не даёт.

Отдельно: README gpu-screen-recorder пишет, что захват через desktop portal на Hyprland был сломан, и указывает фикс в xdg-desktop-portal-hyprland от 2026-08-29, коммит `ba31964ee42b56bcb0d3b78a64ead5d8a1c3c6f6`. Пока в системе старее этот XDPH, автор советует снимать монитор напрямую. Это заявление автора рекордера со ссылкой на коммит, не независимый тест этой машины.

Источник: [README, FAQ «Desktop portal capture doesn't work on Hyprland»](https://git.dec05eba.com/gpu-screen-recorder/plain/README.md).

### 3. Выход, который слышит пользователь, и микрофон одновременно

PipeWire — граф узлов. Поток приложения с `media.class` `Stream/Input/Audio` — захват. Устройство воспроизведения — `Audio/Sink`, микрофон — `Audio/Source`. Политика WirePlumber по умолчанию связывает поток захвата с source, а если источников несколько, берёт узел по умолчанию, затем узел с наивысшим `priority.session`.

Источник: [Linking Policy](https://pipewire.pages.freedesktop.org/wireplumber/policies/linking.html).

Монитор выхода — не второй микрофон. На узлах с входными портами (sink и потоки захвата) WirePlumber создаёт monitor-порты. Настройка `node.features.audio.monitor-ports` по умолчанию `true`. Они отдают то, что уходит в узел, не подменяя воспроизведение.

Источник: [Well-known settings, node.features.audio.monitor-ports](https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/settings.html).

Ключ потока `stream.capture.sink` означает «снимать выход sink, а не выход source». Куда линковаться, задаёт `target.object`: `node.name` или `object.serial`. `pw-record --monitor` в режиме записи читает monitor-порты; `--target` принимает `auto`, `0` или serial/имя. Пример модуля loopback в документации PipeWire делает то же самое комментарием: чтобы снять монитор sink, включить `stream.capture.sink` и поставить `target.object` на имя sink. Отдельный loopback-узел для кольца не обязателен: достаточно собственного потока захвата с этими свойствами.

Источники: [pw_keys, PW_KEY_STREAM_CAPTURE_SINK и PW_KEY_TARGET_OBJECT](https://docs.pipewire.org/group__pw__keys.html), [pw-cat, --target и --monitor](https://docs.pipewire.org/page_man_pw-cat_1.html), [module-loopback](https://docs.pipewire.org/page_module_loopback.html), [pipewire-props, target.object и node.exclusive](https://docs.pipewire.org/page_man_pipewire-props_7.html).

Микрофон — второй поток `Stream/Input/Audio` без `stream.capture.sink`, с `target.object` на source по умолчанию либо с автоподключением. Политика линковки не описывает запрета второго потока захвата. `node.exclusive` по умолчанию ложен; исключительный доступ как раз занял бы устройство один.

Умолчания: `wpctl` знает `@DEFAULT_AUDIO_SINK@` и `@DEFAULT_AUDIO_SOURCE@`. `wpctl set-default` запоминает выбор в файле состояния `default-nodes`, и он важнее автоматического приоритета. Скрипты default-nodes хранят `default.configured.audio.sink` / `default.configured.audio.source` и восстанавливают их при появлении metadata.

Источники: [wpctl](https://pipewire.pages.freedesktop.org/wireplumber/man/wpctl.html), [Default Nodes Scripts](https://pipewire.pages.freedesktop.org/wireplumber/scripting/existing_scripts/default_nodes.html).

Портальный fd от `OpenPipeWireRemote` содержит только узлы скринкаста. Микрофон и монитор sink берутся отдельным соединением с PipeWire сессии пользователя. Документация потоков PipeWire диалога разрешения для этого соединения не описывает.

Часы графа по умолчанию: `default.clock.rate = 48000`. Это конфигурируемое значение, не закон.

Источник: [pipewire.conf](https://docs.pipewire.org/page_man_pipewire_conf_5.html).

Смена устройства по ходу сессии автоматической не является. `target.object` указывает конкретный узел. Заметка в pipewire-props: если поток должен появляться и исчезать вместе с целью, это скрипт WirePlumber, а не свойство потока. Рестарт PipeWire существующий клиент сам не переживает: автор gpu-screen-recorder в закрытом тикете пишет, что повторное подключение после рестарта аудиосервера не поддерживается и программу нужно запустить снова.

Источник: [gpu-screen-recorder-issues #107](https://github.com/dec05eba/gpu-screen-recorder-issues/issues/107).

Юнит gpu-screen-recorder передаёт два `-a` (`AUDIO_DEVICE` и `SECONDARY_AUDIO_DEVICE`). В примере сервиса второй пуст, первый — `default_output`. README показывает запись монитора вместе с выходом по умолчанию: `-a default_output`. Список устройств даёт `--list-audio-devices`. Это подтверждение интерфейса «выход и второй источник в одном процессе», механизм под ним — PipeWire, описанный выше.

### 4. Цена кольца 10–20 минут при 60 fps

Протоколы захвата видео не кодируют. Они кладут один несжатый кадр в SHM или DMA-BUF. Формат `wl_shm` `xrgb8888` в ядре Wayland — 32-битный, 4 байта на пиксель до выравнивания stride, которое протокол screencopy сообщает отдельно. Оценка ниже — без padding и без того, что DMA-BUF может быть другим fourcc.

Источник формата: [wayland.xml](https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/protocol/wayland.xml).

Оценка сырого видео, 4 байта/пиксель, ровно 60 кадров/с, кадр каждый тик, без сжатия:

| Кадр | Байт на кадр | 10 минут | 20 минут |
| --- | ---: | ---: | ---: |
| 1920×1080 | 8 294 400 | ~278 ГиБ | ~556 ГиБ |
| 2560×1440 | 14 745 600 | ~494 ГиБ | ~989 ГиБ |
| 3840×2160 | 33 177 600 | ~1,09 ТиБ | ~2,17 ТиБ |

Считать так: `ширина × высота × 4 × 60 × секунды`, секунды = 600 или 1200, затем деление на 1024³. Это порядок величины при названном допущении. Такое кольцо в RAM не живёт. На диске это непрерывная запись в сотни гигабайт, и вики рекордера отдельно советует SSD уже для сжатого файла.

Рабочий набор сырых кадров при этом маленький: протокол держит один кадр на сессию, клиент может крутить несколько буферов. Четыре кадра 4K по 4 байта — около 126 МиБ. Дорогой становится не пул захвата, а хранение истории.

Звук в PCM при `default.clock.rate` 48000, стерео, два потока (монитор выхода и микрофон), 10 минут:

- S16: `48000 × 2 × 2 × 2 × 600` = 230 400 000 байт, около 220 МиБ; 20 минут — около 440 МиБ.
- F32: вдвое больше, около 440 МиБ и 880 МиБ.

Это влезает в память. Узкое место — видео.

Закодированное кольцо — то, что первичные источники показывают как рабочую схему. README gpu-screen-recorder: replay хранит «последние несколько минут», буфер по умолчанию лежит в RAM как закодированное видео, слишком большую длительность и качество туда класть не следует; `-replay-storage disk` переносит буфер на диск; для предсказуемого размера в replay советуют CBR, примеры `-bm cbr -q 20000` и в FAQ `-bm cbr -q 30000`. Поставленный systemd-юнит ставит `REPLAYDURATION=60`, `FRAMERATE=60`, `BITRATE_MODE=cbr`, `QUALITY=40000`, контейнер mp4, аудиокодек opus. Единицу числа `-q` README в этих абзацах не называет. Если читать 20000 и 40000 как килобиты в секунду, 10 минут занимают примерно 1,4 ГиБ и 2,8 ГиБ, 20 минут — 2,8 ГиБ и 5,6 ГиБ (`бит/с × секунды / 8`). Это арифметика при явном допущении о единице, не измерение.

Кодеки, которые этот рекордер сам перечисляет: видео H264 (по умолчанию), HEVC, AV1, VP8, VP9; звук Opus (по умолчанию), AAC, MP3. Он пишет, что работает на AMD, Intel и NVIDIA, и перечисляет разные runtime-зависимости: VA-API (mesa / intel-media-driver) для AMD и Intel, CUDA и NVENC для NVIDIA, плюс экспериментальное Vulkan-кодирование (`-k h264_vulkan` и аналоги для hevc/av1). Это матрица одной программы, не обнаружение GPU этой машины. Прямой захват монитора у неё на Wayland завязан на KMS и root; для процесса без root её собственный текст оставляет портал.

Источник: [README gpu-screen-recorder](https://git.dec05eba.com/gpu-screen-recorder/plain/README.md), [service](https://git.dec05eba.com/gpu-screen-recorder/plain/extra/gpu-screen-recorder.service).

Кодировщик без привязки к производителю GPU, из документации FFmpeg: libx264 кодирует H.264, если FFmpeg собран с `--enable-libx264`. Это программный кодировщик библиотеки x264.

Источник: [FFmpeg general, раздел x264](https://www.ffmpeg.org/general.html).

wf-recorder вики Hyprland приводит как лёгкий рекордер Wayland, пишущий в файл (`wf-recorder -f …`). Кольца он в цитируемых страницах не описывает. Он годится как свидетельство, что пользовательский процесс на Hyprland пишет закодированный файл с выхода, и как пример клиента права `screencopy`. Не как измерение 20-минутного кольца.

Измерения «10–20 минут, 60 fps, Hyprland, этот GPU, без пропусков» в первичных источниках нет. Жизнеспособность следует из размера сырого кадра, из того, что протоколы отдают кадр по одному, и из того, что закодированный replay в RAM уже существует как режим отдельной программы. Выдержит ли конкретная машина 20 минут при 60 fps программным x264 или аппаратным кодировщиком, документы не говорят.

XDPH отдельно предупреждает, что SHM медленнее DMA-BUF на высоком разрешении. Для длинного кольца это про стоимость копирования каждого кадра в кодировщик, не про размер уже сжатого буфера.

## Что спека может утверждать

- После входа отдельный процесс пользователя, запущенный из `hyprland.start` / `exec-once` или из systemd user unit сессии, может без своего окна начать копировать кадры одного `wl_output`.
- На Hyprland, где принуждение прав оставлено выключенным, этот прямой захват не показывает диалог. Если принуждение включено, диалог убирает правило `screencopy` / `allow` на путь бинаря; иначе право по умолчанию спрашивает.
- На Hyprland 0.54 и новее для того же кадра выхода есть `ext-image-copy-capture-v1`. `zwlr_screencopy_manager_v1` в том же релизе остаётся. Оба отдают несжатый буфер, SHM всегда в контракте wlr-screencopy до версии 2 включительно, DMA-BUF — когда композитор его объявил.
- Портал ScreenCast умеет отдать ровно один поток монитора в PipeWire. Тихий старт со второго раза возможен только при `persist_mode = 2`, сохранённом приложением новом токене, том же `app_id` и всё ещё существующем выходе. Первый запуск проходит через выбор.
- Системный выход и микрофон по умолчанию снимаются двумя потоками захвата PipeWire одновременно: `stream.capture.sink` плюс `target.object` на sink по умолчанию, и отдельный поток на source по умолчанию. Умолчания WirePlumber помнит между перезапусками. Исключительный режим устройства для этого не нужен и по умолчанию выключен.
- Кольцо 10–20 минут при 60 fps хранится закодированным потоком (RAM или диск). Сырые кадры рабочего стола такого срока в кольцо не кладутся. Звук в PCM на этот срок — сотни мегабайт и кольцо не ограничивает.
- 60 fps — темп, с которым приложение забирает или прореживает кадры. Верхняя граница сессии портала настраивается (`max_fps`).

## Что остаётся открытым

- Какой выход считать «primary». Ни screencopy, ни xdg-output в прочитанных страницах, ни вики мониторов Hyprland не задают Wayland-флаг primary. Правило умолчания (коннектор из конфига, монитор в позиции `0x0`, focused на момент старта) — решение продукта. `xrandr --primary` влияет на X11-клиентов.
- Версия Hyprland и xdg-desktop-portal-hyprland на этой установке CachyOS. От неё зависит, есть ли image-copy-capture (с 0.54.0) и починен ли портальный захват, на который ссылается gpu-screen-recorder (коммит XDPH от 2026-08-29).
- Стабильный `app_id` у процесса, поднятого голым `exec_cmd` без `.desktop` и без `Registry.Register`. Без него `persist_mode = 2` может не найти токен на следующем входе. Для прямого протокола этот вопрос не стоит.
- Частота popup `ask` внутри одного долгого захвата при включённых правах. И точное имя ключа права для image-copy-capture: вики документирует `screencopy` для захвата мимо портала, отдельной строки про новый протокол нет.
- Жизнь процесса при выходе Hyprland. Сокет захвата при этом пропадает. Переживает ли PID сигнал композитора, вики не пишет. Содержимое RAM-кольца рестарт композитора не сохраняет; диск — отдельное решение.
- Следование за сменой sink/source по умолчанию и за рестартом PipeWire. Зафиксированный `target.object` сам за новым устройством не идёт. Длинное кольцо может потерять звук, пока процесс не переподключится.
- Какой кодировщик реально есть на машине. libx264 задокументирован как программный H.264 при соответствующей сборке FFmpeg. VA-API, NVENC и Vulkan Video перечислены как зависимости конкретной программы под конкретные драйверы. Ни один источник не измерял 20 минут 60 fps на неуказанном GPU.
- Единица `-q` у gpu-screen-recorder в режиме CBR. В README это величина битрейта, но «килобиты» там не написаны. Цифры 1,4–5,6 ГиБ выше верны только если 20000 и 40000 — это кбит/с.
- Курсор, HDR, дробный scale, повреждённые области против полного кадра, поведение при отключении монитора. Протоколы это умеют по-разному, продукт этого не фиксировал.
- Прямой KMS-захват монитора, как его делает gpu-screen-recorder без портала. Он требует root. Для обычного процесса сессии это не путь.

## Источники

- https://wayland.app/protocols/wlr-screencopy-unstable-v1
- https://wayland.app/protocols/ext-image-copy-capture-v1
- https://wayland.app/protocols/ext-image-capture-source-v1
- https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/protocol/wayland.xml
- https://github.com/hyprwm/Hyprland/releases/tag/v0.54.0
- https://github.com/hyprwm/Hyprland/releases/tag/v0.54.3
- https://github.com/hyprwm/Hyprland/pull/11709
- https://wiki.hypr.land/Configuring/Advanced-and-Cool/Permissions/
- https://wiki.hypr.land/Useful-Utilities/Screenshots-and-Recording/
- https://wiki.hypr.land/Hypr-Ecosystem/xdg-desktop-portal-hyprland/
- https://wiki.hypr.land/Configuring/Basics/Autostart/
- https://wiki.hypr.land/Useful-Utilities/Systemd-start/
- https://wiki.hypr.land/0.53.0/Configuring/Monitors/
- https://wiki.hypr.land/IPC/
- https://wiki.hypr.land/FAQ/
- https://github.com/hyprwm/hyprland-wiki/blob/5e1782bd/content/Configuring/Keywords.md
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html
- https://github.com/flatpak/xdg-desktop-portal/blob/main/desktop-portal/xdp-session-persistence.c
- https://github.com/flatpak/xdg-desktop-portal/blob/1.20.0/NEWS.md
- https://github.com/hyprwm/xdg-desktop-portal-hyprland/compare/v1.3.11...v1.3.12
- https://github.com/hyprwm/xdg-desktop-portal-hyprland/releases/tag/v1.3.5
- https://github.com/hyprwm/xdg-desktop-portal-hyprland/issues/350
- https://docs.pipewire.org/group__pw__keys.html
- https://docs.pipewire.org/page_man_pw-cat_1.html
- https://docs.pipewire.org/page_module_loopback.html
- https://docs.pipewire.org/page_man_pipewire-props_7.html
- https://docs.pipewire.org/page_man_pipewire_conf_5.html
- https://pipewire.pages.freedesktop.org/wireplumber/policies/linking.html
- https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/settings.html
- https://pipewire.pages.freedesktop.org/wireplumber/man/wpctl.html
- https://pipewire.pages.freedesktop.org/wireplumber/scripting/existing_scripts/default_nodes.html
- https://git.dec05eba.com/gpu-screen-recorder/plain/README.md
- https://git.dec05eba.com/gpu-screen-recorder/plain/extra/gpu-screen-recorder.service
- https://github.com/dec05eba/gpu-screen-recorder-issues/issues/107
- https://www.ffmpeg.org/general.html
