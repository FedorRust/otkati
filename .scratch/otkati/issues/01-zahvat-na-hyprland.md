# Захват монитора и звука на Hyprland

Label: wayfinder:research
Type: research
Status: resolved
Blocked by:

## Question

Как на Hyprland (CachyOS, PipeWire) отдельная программа может с входа в сессию держать кольцо одного монитора, звука выхода и микрофона — и что из этого реально выдерживает кольцо длиной 10–20 минут?

## Answer

Обычный процесс сессии снимает один `wl_output` прямым протоколом (wlr-screencopy, а с Hyprland 0.54 ещё и ext-image-copy-capture) без диалога, пока права Hyprland не включены; портал ScreenCast даёт один PipeWire-поток, но первый раз спрашивает пользователя, а дальше молчит только при сохранённом restore token. Выход и микрофон — два потока захвата PipeWire одновременно: монитор sink по умолчанию и source по умолчанию. Сырые 60 fps на 10–20 минут — сотни гигабайт даже в 1080p, такое кольцо держат закодированным. «Primary» и гарантия ровно 60 уникальных кадров протоколами не заданы.

Находки: [zahvat-na-hyprland.md](../research/zahvat-na-hyprland.md)
