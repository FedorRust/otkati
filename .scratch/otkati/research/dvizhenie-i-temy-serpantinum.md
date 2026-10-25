# Первый запуск и темы Serpantinum

Срез: ветка `master`, коммит [`92e4325ad64b1a9154e3e9095c67dfc6232ef0c1`](https://github.com/ilyamiro/serpantinum/commit/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1) (2026-10-25, «fix(main): ignore repeated close requests so the window can't get stuck open but undrawn»). Ниже только этот исходник. Документация serpantinum.dev не сверялась.

## Вопрос

Как устроены первый запуск Serpantinum и каталог тем в `src/assets/themes`, включая Matugen? Нужны конкретный жест входной анимации и контракт темы.

## Короткий ответ

Первый запуск оболочки — это не открытие настроек. Если нет файла `~/.local/state/serpantinum/first_launch.done`, оболочка один раз показывает полноэкранный оверлей `src/quickshell/serp/Start.qml` (смывание экрана, набор текста, заливка логотипа). Флаг создаётся до анимации. По закрытии оверлея скрипт открывает настройки на вкладке Welcome.

Вход самой панели настроек — другой жест, в `src/quickshell/guide/GuidePopup.qml`. В обычном режиме (настройки внутри оболочки, это значение по умолчанию) панель проявляется и чуть увеличивается, сайдбар выезжает слева, пункты вкладок выезжают со сдвигом по индексу, содержимое поднимается снизу. Этот жест повторяется при каждом показе панели. Отдельного «проиграть вход настроек только один раз» в оболочке нет.

Каталог `src/assets/themes` на этом коммите — 96 файлов `*.json`. В список попадают все JSON, которые парсятся: 94 статические темы с одним и тем же набором из 22 цветов, плюс `Matugen.json` (`{"name":"Matugen","isMatugen":true}`). Пустой `Nightfox.json` отбрасывается. Активная тема читается не из каталога, а из `theme` в `settings.json`: при `matugen: true` цвета берёт Matugen с обоев.

## Первый запуск

Цепочка файлов:

1. `src/quickshell/Shell.qml` в `Component.onCompleted` вызывает `FirstLaunch.checkFirstLaunch()`. Тот же вызов есть в `Component.onCompleted` у `src/quickshell/singletons/system/FirstLaunch.qml`. Второй вызов ничего не делает: `executed` уже true.
2. Через 350 мс (`startupDelayTimer`) запускается `src/scripts/first_launch.sh --check`.
3. Скрипт смотрит `$QS_STATE_DIR/first_launch.done`, по умолчанию `~/.local/state/serpantinum/first_launch.done`. Файл есть — печать `SKIP` и выход. Файла нет — `touch` флага сразу, затем строка `FIRST|<случайные обои>|<путь к Start.qml>`.
4. Обои: если в настройках задан `wallpaperDir` или `wallpaper_dir` и каталог существует, `find` + `shuf -n 1` среди png/jpg/jpeg/webp/mp4/mkv/mov/webm. Иначе путь пустой.
5. `FirstLaunch.runSetup` при непустом пути вызывает `Matugen.generate(wp, "", "")` и `Wallpaper.setWallpaper("all", wp, "fade")`, затем ждёт 900 мс. Без обоев ждёт 100 мс. После таймера грузит `src/quickshell/serp/Start.qml` (или путь из скрипта) и вызывает `item.start()` / `item.play()`, если такой функции нет — оверлей всё равно стартует сам.
6. Сигналы `firstLaunchStarted` и `startAnimations` никем не слушаются. Анимацию запускает сам `Start.qml`: через 100 мс `introChoreography`.
7. По `finished` или `closed` снова вызывается `first_launch.sh --open-guide`, а тот — `qs_manager.sh open settings welcome`.

Оверлей — `PanelWindow` на весь экран, слой Overlay, namespace `welcome-guide`, клавиатура exclusive. Это не окно настроек. Жест, пока пользователь не нажмёт кнопку или Escape:

- фон: `panelReveal` 0→1 за 2000 мс, `Easing.InOutCubic`, шейдер `src/assets/shaders/effects/screen_wipe.frag.qsb`;
- набор строки welcome за 850 мс (`Easing.InOutSine`), затем погасание за 400 мс;
- набор слогана за 1600 мс и масштаб текста 0.94→1 (`Easing.OutExpo`), имя автора всплывает за 500 мс (`Easing.OutQuint`);
- логотип: непрозрачность и масштаб 0.94→1, заливка `logoFillLevel` до 1.5 за 4500 мс (`Easing.OutSine`) шейдером `src/assets/shaders/fluid/logo_water_wave.frag.qsb`, «монитор» с масштаба 1.5 до 1 за 2800 мс (`Easing.OutQuint`);
- кнопка Start проявляется за 600 мс. Клик или Escape: непрозрачность сцены до 0 за 400 мс (`Easing.OutQuint`) и обратное смывание `panelReveal` до 0 за 1760 мс (`Easing.InOutCubic`).

Дальше открываются настройки, не оверлей.

`src/scripts/settings.sh`: вкладка пишется в кэш `guide/last_tab.txt` только на пути отдельного окна. По умолчанию отдельное окно выключено (`general.settingsSeparateWindow` нет в `config/serpantinum/settings.json`, в `GeneralTab.qml` запасное значение false, в скрипте `jq` тоже `// false`). Тогда IPC `embeddedSettings` в `src/quickshell/Main.qml` вызывает `switchWidget("guide", "welcome")`. Виджет — `guide/GuidePopup.qml` из `src/quickshell/WindowRegistry.js`: 1200×750, по центру, `draggable: true`. Приход из скрытого состояния не морфит окно мастера (`disableMorph = true`); морф 300 мс `Easing.OutCubic` бывает только при переключении с другого виджета.

Вкладка по умолчанию в `GuidePopup` — индекс 0, Welcome (`WelcomeTab.qml`). Команда первого запуска явно передаёт `welcome`. Позже `FileView` на `last_tab.txt` может переключить вкладку, если файл уже есть. Отдельной вкладки «длина и качество» в оболочке нет.

### Вход панели настроек

`GuidePopup.resetAndPlayIntro()` вызывается каждый раз, когда `visible` становится true, и ещё раз из `Component.onCompleted`, если панель уже видима. Флага «уже проигрывали» нет. При скрытии intro-свойства сбрасываются в 0, поэтому следующий показ начинает сначала.

Встроенная панель (`standalone: false`, обычный путь) — `shellStartupSequence`:

| Что | Свойство | Жест | Время |
| --- | --- | --- | --- |
| Вся панель | `introBase` | непрозрачность 0→1 и масштаб 0.95→1, `Easing.OutExpo` | 380 мс |
| Сайдбар | `introSidebar` | непрозрачность 0→1 и сдвиг по X с −30 px до 0 (в RTL с +30), `Easing.OutCubic`, пауза 30 мс | 240 мс |
| Пункты вкладок | `introTabs` | свойство 0→1 линейно за 320 мс после паузы 50 мс; у пункта `index` старт сдвинут на `index * 0.04` | см. ниже |
| Контент | `introContent` | непрозрачность 0→1, масштаб 0.95→1, сдвиг по Y с 20 px до 0, `Easing.OutCubic`, пауза 90 мс | 360 мс |

Сдвиги заданы как `popup.s(30)`, `popup.s(24)`, `popup.s(20)`. `Scaler.s` в `src/quickshell/singletons/theme/Scaler.qml` на этом коммите возвращает аргумент как есть, без умножения на `uiScale`.

Пункт вкладки (`GuideSidebar.qml`): непрозрачность `(introTabs - index*0.04) / 0.28`, ограниченная 0…1; сдвиг по X на 24 px домножается на прогресс. Прогресс пункта — не именованный easing Qt, а формула при `c1 = 0.85`, `c3 = 1.85`, `p` в 0…1:

`1 + c3*(p-1)^3 + c1*(p-1)^2`, где `p = (introTabs - index*0.04) / 0.42`.

Отдельное окно настроек (`SettingsApp.qml`, `standalone: true`) этот каскад не играет. `introSidebar`, `introContent` и `introTabs` сразу 1. Анимируется только `introBase`: непрозрачность 0→1 за 160 мс, `Easing.OutCubic`. Масштаб панели остаётся 1. Окно всё равно создаётся заново при каждом запуске (`visible: false` у окна делает `Qt.quit()`), поэтому короткий fade повторяется на каждый запуск процесса, но не хранится как «первый раз».

Пока открыта вкладка Welcome, `WelcomeTab.qml` дополнительно гоняет заливку логотипа: `logoFillLevel` 0→1.15 за 6500 мс, `Easing.OutCubic`, при каждом показе вкладки. Это содержимое вкладки, не вход окна.

Закрытие встроенной панели: контент и сайдбар за 100 мс `Easing.InExpo`, вкладки за 80 мс `Easing.InQuad`, затем непрозрачность панели за 130 мс `Easing.InQuart`. У отдельного окна гаснет только панель, 110 мс `Easing.InQuad`.

## Контракт темы

Каталог на диске и список в UI — не одно и то же.

В `src/assets/themes` 96 файлов, все с суффиксом `.json`, других файлов нет. Обход делает `src/quickshell/guide/theme/theme_sorter.py` (`process_dir`): каждый `*.json` в системном каталоге и в пользовательском `~/.local/state/serpantinum/themes` (или `$QS_STATE_DIR/themes`). Файл, который не парсится, пропускается. `Nightfox.json` пустой (0 байт), в список не входит. Итого из системного каталога в выдачу попадают 95 объектов: 94 статические темы и Matugen. Сортировщик дописывает `category` (`system` / `user`), пользовательским ещё `isCustom: true`, и вставляет строки `{"isDivider": true}` между группами. Порядок не алфавитный и не порядок каталога: Matugen отдельно (`name` в нижнем регистре равен `matugen`), свои темы отдельно, остальные по светлоте и корзине оттенка. `ThemeTabState.reloadThemes` зовёт этот скрипт и делит ответ на `systemPresets` и `userPresets`.

Статическая тема, общий случай — `src/assets/themes/Mocha.json`. Ровно `name` и `colors` из 22 ключей. Те же 22 ключа у всех 94 разобранных статических файлов, и они совпадают с полями, которые пишет `ThemeBackend.applyColorObject`:

`base`, `mantle`, `crust`, `text`, `subtext0`, `subtext1`, `surface0`, `surface1`, `surface2`, `overlay0`, `overlay1`, `overlay2`, `blue`, `sapphire`, `peach`, `green`, `red`, `mauve`, `pink`, `yellow`, `maroon`, `teal`.

Значения Mocha совпадают с начальными цветами в `ThemeBackend.qml` (`base: #1e1e2e` и далее).

У 52 файлов (каменные имена, например `Alexandrite.json`) есть ещё `isColorFocused: true`. Его читает только сортировщик, чтобы считать оттенок по акцентам, а не по фону. `ThemeBackend` это поле не применяет.

Валидатора схемы нет. Нет ключа в `colors` — `applyColorObject` оставляет уже стоящий цвет. Тема без `name` получит пустое имя при сортировке, но в каталоге таких нет.

Выбор пресета (`ThemeTabState.applyPreset`) пишет в объект `theme` настроек `activePreset` и `matugen`. Для обычной темы ещё копирует `colors` в настройки. Для Matugen цвета в настройки не копирует.

Живой объект настроек — `Config.getSetting("theme", …)`, файл `~/.config/serpantinum/settings.json` (`src/scripts/config.sh`, ключ `QS_SETTINGS`). Запасной объект в `ThemeTabState.defaultThemeSettings`: `fontFamily`, `borderRadius`, `elementOpacity: 100`, `blurBackground: false`, `blurStyle: "blur"`, `activePreset: "Matugen"`, `matugen: true`, `colors: {}`. Поставленный `config/serpantinum/settings.json` задаёт шрифт `Adwaita Mono`, `borderRadius: 10`, `activePreset: "Matugen"`, `matugen: true`. Шрифт, радиус, непрозрачность и размытие живут в настройках, не в JSON пресета.

Оболочка считает тему Matugen, если `theme.matugen === true`. Если поля `matugen` нет, срабатывают запасные сравнения `theme` / `name` / `currentTheme` / `current` с `"matugen"`, либо `activePreset === "Matugen"`, либо `activePreset` вообще не задан (`Matugen.qml` и `ThemeBackend.qml`, функция `isMatugenTheme`). То есть отсутствие записи о теме тоже означает Matugen.

Дальше `ThemeBackend.updateColors`:

- Matugen: применить уже прочитанный `qs_matugen_colors.json`; если это не отдельное окно настроек, вызвать `Matugen.generate` с путём `theme.wallpaper` или настройки `wallpaper`.
- Не Matugen: если в настройках есть `colors`, `palette` или сам объект с полем `base` — применить его и (вне standalone) прогнать `generateFromStatic`. Иначе перечитать `qs_colors.json`.

Путь цветов: `QS_COLORS_JSON` или `~/.local/state/serpantinum/qs_colors.json`. Файл Matugen: `~/.local/state/serpantinum/qs_matugen_colors.json`. Оба `FileView`. У Matugen-файла `watchChanges: true`. У `qs_colors.json` слежение выключено, его перечитывают явно (`reload`), в том числе по сигналу `Matugen.generationFinished`.

## Matugen

`src/assets/themes/Matugen.json` целиком:

```json
{
  "name": "Matugen",
  "isMatugen": true
}
```

Цветов в файле нет. Карточка в списке тем берёт их из последнего `qs_matugen_colors.json` (`ThemeTabState`, `prevMatugen*`), а не из пресета.

Генерация с обоев — `Matugen.generate` в `src/quickshell/singletons/theme/Matugen.qml`. Пустой путь или тема не Matugen — выход false. Иначе процесс:

`nice -n 15 matugen image <путь> -c <assets/matugen/config.toml> -m <mode> -t <schemeType> --source-color-index 0`

`mode` по умолчанию `"dark"`, `schemeType` по умолчанию `"scheme-tonal-spot"`, если их не передали и нет в `theme`. `config.toml` пишет шаблон `templates/serpantinum_matugen_colors.json.template` в `~/.local/state/serpantinum/qs_colors.json`. После успешного image-запуска файл копируется в `qs_matugen_colors.json`. Шаблон кладёт роли Material в те же 22 ключа. Несколько ключей делят один источник: `overlay1` и `overlay2` оба из `inverse_surface`, `mauve` из `primary`, `teal` из `secondary`.

Кто вызывает генерацию с картинки:

- смена обоев в `src/quickshell/wallpaper/WallpaperEngine.qml`: для картинки `Matugen.generate(путь файла)`, для видео сначала кадр ffmpeg в `wpSnapshotPath`, затем `generate` уже со снимка;
- выбор обоев в `WallpaperPicker.qml`;
- первый запуск, если нашёлся случайный файл: `FirstLaunch` вызывает `generate` с этим файлом напрямую (для видео отдельного снимка здесь нет), и следом `setWallpaper(..., "fade")`, а движок обоев на `onWallpaperChanged` вызывает `changeWallpaper`, который генерирует ещё раз, для видео уже со снимка;
- `ThemeBackend.updateColors`, если в настройках есть путь обоев.

Повтор того же image-запроса, пока процесс ещё идёт или сразу после успеха, не стартует второй `matugen`.

Статическая тема всё равно зовёт бинарь `matugen`, но входом служит JSON, а не обои: `generateFromStatic` пишет цвета в `qs_colors.json` и гоняет `matugen json` с `config-static.toml`. Этот конфиг не перезаписывает `qs_colors.json` шаблоном оболочки (в нём только kitty и cava). Цвета панели при этом уже применены из объекта `colors` до вызова бинаря.

## Что спека может утверждать

- Первый показ настроек внутри оболочки (режим по умолчанию) играет жест из таблицы выше: панель 380 мс OutExpo, непрозрачность и масштаб 0.95→1; сайдбар с −30 px за 240 мс OutCubic после 30 мс; вкладки каскадом по формуле `getTabProgress` / `getTabOpacity`; контент с +20 px и масштаба 0.95 за 360 мс OutCubic после 90 мс. Это язык движения окна настроек, и после него панель остаётся на текущей вкладке.
- Окно настроек по центру, 1200×750, пока его не перетащат. Вкладка первого программного открытия после оверлея — Welcome.
- Статическая тема для копирования — `name` плюс 22 ключа `colors`, как в `Mocha.json`. `isColorFocused` на покраску не влияет.
- Matugen — маркер `isMatugen: true` без палитры. Активность — `settings.json` → `theme.matugen` и `theme.activePreset`. Цвета окна — 22 ключа из `qs_matugen_colors.json`, снятые `matugen image` с обоев, режим `dark`, схема `scheme-tonal-spot`, если свои не заданы.
- Список тем — разбор всех `*.json` каталога плюс пользовательский каталог, а не зашитый массив. На этом коммите в `src/assets/themes` 96 файлов.

Решение продукта «проиграть вход окна один раз и больше не повторять» спека может зафиксировать у себя. В оболочке такого флага для входа панели нет: повторяется каждый показ.

## Чего в оболочке нет

- Нет одного жеста, который и есть «первый запуск», и есть «вход окна настроек». Оверлей `Start.qml` уходит по кнопке или Escape и только потом открывает настройки. Его нельзя описать как вход окна, которое остаётся на регуляторах.
- Нет флага «вход панели настроек уже сыгран». `first_launch.done` гасит только оверлей, и ставится до того, как оверлей доиграет. Сбой посреди `Start.qml` при следующем старте оверлей уже не покажет.
- Нет каскада сайдбара в отдельном окне настроек. Там только fade непрозрачности 160 мс, масштаб 1.
- `startAnimations` / `firstLaunchStarted` никуда не подключены. Отдельной intro-анимации «кроме открытия настроек на Welcome» не отсутствует: оверлей есть, но это не вкладка Welcome. Заливка логотипа на Welcome — третья, вкладковая анимация, 6500 мс, на каждый показ вкладки.
- Нет схемы, которая отвергает тему с неполным `colors`. Нет в каталоге цветов у Matugen. Пустой `Nightfox.json` не тема.
- Нет вкладки длины и качества и нет отдельной красной плашки «нет места», которая выключена из темы. Красный `ThemeBackend.red` — обычный цвет палитры (`#f38ba8` у Mocha) и красится вместе с темой.

## Источники

Коммит `92e4325ad64b1a9154e3e9095c67dfc6232ef0c1`.

- [src/quickshell/Shell.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/Shell.qml) — вызов проверки первого запуска.
- [src/quickshell/singletons/system/FirstLaunch.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/singletons/system/FirstLaunch.qml) — таймеры, загрузка `Start.qml`, `--open-guide`.
- [src/scripts/first_launch.sh](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/scripts/first_launch.sh) — флаг, случайные обои, `open settings welcome`.
- [src/quickshell/serp/Start.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/serp/Start.qml) — полноэкранный жест.
- [src/scripts/settings.sh](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/scripts/settings.sh) и [src/scripts/qs_manager.sh](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/scripts/qs_manager.sh) — открытие настроек.
- [src/quickshell/Main.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/Main.qml) — `embeddedSettings`, `switchWidget`.
- [src/quickshell/WindowRegistry.js](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/WindowRegistry.js) — размер и компонент `guide`.
- [src/quickshell/guide/GuidePopup.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/GuidePopup.qml) — вход панели и вкладка Welcome.
- [src/quickshell/guide/GuideSidebar.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/GuideSidebar.qml), [src/quickshell/guide/GuideContent.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/GuideContent.qml) — сдвиги.
- [src/quickshell/singletons/theme/Scaler.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/singletons/theme/Scaler.qml) — `s()` без масштаба.
- [src/quickshell/SettingsApp.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/SettingsApp.qml) — отдельное окно.
- [src/quickshell/guide/WelcomeTab.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/WelcomeTab.qml) — заливка логотипа вкладки.
- [src/quickshell/guide/theme/theme_sorter.py](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/theme/theme_sorter.py) и [ThemeTabState.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/guide/theme/ThemeTabState.qml) — каталог и применение пресета.
- [src/assets/themes/Mocha.json](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/themes/Mocha.json), [Alexandrite.json](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/themes/Alexandrite.json), [Matugen.json](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/themes/Matugen.json), пустой [Nightfox.json](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/themes/Nightfox.json).
- [src/quickshell/singletons/theme/ThemeBackend.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/singletons/theme/ThemeBackend.qml), [Matugen.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/singletons/theme/Matugen.qml).
- [src/assets/matugen/config.toml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/matugen/config.toml) и [templates/serpantinum_matugen_colors.json.template](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/assets/matugen/templates/serpantinum_matugen_colors.json.template).
- [src/quickshell/wallpaper/WallpaperEngine.qml](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/src/quickshell/wallpaper/WallpaperEngine.qml) — снимок видео перед `matugen image`.
- [config/serpantinum/settings.json](https://github.com/ilyamiro/serpantinum/blob/92e4325ad64b1a9154e3e9095c67dfc6232ef0c1/config/serpantinum/settings.json) — поставленный пресет Matugen.
