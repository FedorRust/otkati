# Otkati

Instant replay for Hyprland. The ring keeps the last minutes of one monitor, the desktop audio, and the microphone. A bind saves an mp4 from that ring.

The default bind is Alt+Z, the folder is ~/Videos/Откаты, and a file is named otkat1-2026-10-26.mp4. The window starts in English. Russian is a switch in the settings.

Requirements
A Hyprland session.
An NVIDIA or AMD card, and gpu-screen-recorder.
Qt 6: /usr/lib/qt6/bin/qml.
Rust and cargo.
python3, for the first question about which screen to capture.
The first launch asks for the screen through the system dialog and remembers the answer. Until there is an answer, no picture enters the ring.

Start
From the repository root:


./otkati
The command builds otkati-ring if the binary is missing, installs the user services otkati-ring and otkati-face, and starts them. The next graphical login starts them on its own. Running ./otkati again while the services are up does not restart the ring.

If ~/.cargo/bin is on PATH, the same command is available as otkati.

The settings window opens with the face. It has length, quality, encoder, ring place, monitor, language, theme, otkat folder, and bind.

The live ring records H.264, 60 frames per second, about 10 minutes, with the cursor, desktop audio, and the microphone. The length, bitrate, and encoder controls are in the window, but they do not reach this recording yet.

Bind
The face waits for a 1 in /tmp/otkati-bind. For Hyprland:


bind = ALT, Z, exec, sh -c 'printf 1 > /tmp/otkati-bind'
After that, Alt+Z asks the ring to save an otkat and shows a short card on the ring monitor. Focus stays in the game.

# Откаты

Программа мгновенного повтора для Hyprland. Кольцо держит последние минуты одного монитора, звука выхода и микрофона. По бинду из кольца сохраняется mp4.

По умолчанию бинд `Alt+Z`, папка откатов `~/Videos/Откаты`, имя файла `otkat1-2026-10-26.mp4`. Язык окна английский, в настройках его можно сменить на русский.

## Что нужно

- Сессия Hyprland.
- Карта NVIDIA или AMD и `gpu-screen-recorder`.
- Qt 6: `/usr/lib/qt6/bin/qml`.
- Rust и `cargo`.
- `python3` для первого вопроса, какой экран отдать.

Первый запуск спрашивает экран через системный диалог и запоминает ответ. Пока ответа нет, картинка в кольцо не попадает.

## Запуск

Из корня репозитория:

```sh
./otkati
```

Команда собирает `otkati-ring`, если бинарника ещё нет, ставит пользовательские службы `otkati-ring` и `otkati-face` и поднимает их. Следующий вход в графическую сессию поднимает их сам. Повторный `./otkati`, пока службы живы, кольцо не перезапускает.

Если `~/.cargo/bin` есть в `PATH`, та же команда доступна как `otkati`.

Окно настроек открывается вместе с лицом. В нём длина, качество, кодировщик, место кольца, монитор, язык, тема, папка откатов и бинд.

Живое кольцо пишет H.264, 60 кадров в секунду, около 10 минут, с курсором, звуком выхода и микрофоном. Ручки длины, битрейта и кодировщика в окне есть, но до этой записи они пока не доходят.

## Бинд

Лицо ждёт единицу в `/tmp/otkati-bind`. Для Hyprland:

```
bind = ALT, Z, exec, sh -c 'printf 1 > /tmp/otkati-bind'
```

После этого `Alt+Z` просит кольцо сохранить откат и показывает плашку на мониторе кольца. Фокус остаётся в игре.
