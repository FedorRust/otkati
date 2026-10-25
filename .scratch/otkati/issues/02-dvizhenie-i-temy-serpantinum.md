# Движение первого запуска и темы Serpantinum

Label: wayfinder:research
Type: research
Status: resolved
Blocked by:

## Question

Как устроены первый запуск Serpantinum и каталог тем в `src/assets/themes`, включая Matugen? Спеке нужен конкретный жест входной анимации и контракт темы, а не общее впечатление от оболочки.

## Answer

Первый запуск оболочки — полноэкранный оверлей `Start.qml` (смывание, набор текста, заливка логотипа), один раз по флагу `first_launch.done`, который ставится до анимации. После него настройки открываются на вкладке Welcome. Вход самой панели — другой жест и повторяется при каждом показе: панель 380 мс (непрозрачность и масштаб 0.95→1, OutExpo), сайдбар с −30 px, вкладки каскадом, контент с +20 px. Отдельное окно настроек играет только fade 160 мс. В `src/assets/themes` на `92e4325` 96 json; в список входят все разобранные, пустой `Nightfox.json` отбрасывается. Статическая тема — `name` и 22 ключа `colors` (как Mocha). Matugen — `{"name":"Matugen","isMatugen":true}` без цветов; живые цвета снимает `matugen image` с обоев в `qs_matugen_colors.json`.

Подробности: `.scratch/otkati/research/dvizhenie-i-temy-serpantinum.md`
