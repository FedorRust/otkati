# Как поставить

Программа пишет последние минуты одного монитора, звука выхода и микрофона. Alt+Z сохраняет из этого mp4.

Ты уже должен сидеть в Hyprland. Видеокарта NVIDIA или AMD, драйвер уже установлен. Система с `pacman`: Arch, CachyOS, Manjaro.

## 1. Открой терминал

В Hyprland он открывается своей клавишей. Если не помнишь, открой меню и запусти Kitty, Alacritty или Ghostty.

## 2. Одна команда

Скопируй строку целиком, вставь в терминал, нажми Enter.

```sh
sudo pacman -S --needed --noconfirm git && git clone --branch otkati https://github.com/FedorRust/otkati.git "$HOME/otkati" && "$HOME/otkati/install"
```

Пароль спросит один раз. Пока печатаешь, символы не появляются, так и должно быть.

Первый раз долго. Качаются пакеты, потом собирается Rust. Можно отойти. В конце откроется окно настроек.

Команда ставит git, компилятор, Rust, Qt 6, Python, ffmpeg, gpu-screen-recorder, портал выбора экрана, PipeWire и сам шрифт окна. Программа оказывается в папке `otkati` у тебя дома и сразу запускается.

## 3. Выбери монитор

Система спросит, какой экран писать. Выбери тот, где игра.

Если вопроса не было, выйди из Hyprland и зайди снова. Потом в терминале:

```sh
"$HOME/otkati/install"
```

## 4. Повесь Alt+Z

Без этой строки клавиша ничего не сохраняет. Открой файл, где у тебя остальные бинды. Чаще всего это `~/.config/hypr/hyprland.conf`. Добавь строку:

```
bind = ALT, Z, exec, sh -c 'printf 1 > /tmp/otkati-bind'
```

Сохрани файл и в терминале выполни:

```sh
hyprctl reload
```

Alt+Z сохраняет откат. Файл лежит в `~/Videos/Откаты`, имя вида `otkat1-2026-10-26.mp4`. Фокус остаётся в игре.

Живое кольцо пишет H.264, 60 кадров в секунду, около 10 минут. Длина, битрейт и кодировщик в окне есть, на эту запись они пока не влияют.

## Поставить ещё раз

Папка `~/otkati` уже есть, клонировать заново не нужно:

```sh
"$HOME/otkati/install"
```

# Install

Otkati keeps the last minutes of one monitor, the desktop audio, and the mic. Alt+Z saves that as an mp4.

You need to already be in Hyprland, on Arch, CachyOS, or Manjaro, with an NVIDIA or AMD card and the driver installed.

## 1. Open a terminal

In Hyprland that is its own key. If you do not remember it, open the launcher and start Kitty, Alacritty, or Ghostty.

## 2. One command

Copy the whole line, paste it, press Enter.

```sh
sudo pacman -S --needed --noconfirm git && git clone --branch otkati https://github.com/FedorRust/otkati.git "$HOME/otkati" && "$HOME/otkati/install"
```

It asks for your password once. Nothing shows up while you type. That is normal.

The first run takes a while. It downloads the packages, then builds the Rust part. When it finishes, the settings window opens.

That command installs git, a compiler, Rust, Qt 6, Python, ffmpeg, gpu-screen-recorder, the screen picker, PipeWire, and the window font. The program lands in `otkati` in your home folder and starts.

## 3. Pick the monitor

The system asks which screen to record. Pick the one with the game.

If that question never shows up, log out of Hyprland and back in. Then run:

```sh
"$HOME/otkati/install"
```

## 4. Hook up Alt+Z

The key does nothing until this line is in your Hyprland config. Open the file with your other binds. That is usually `~/.config/hypr/hyprland.conf`. Add:

```
bind = ALT, Z, exec, sh -c 'printf 1 > /tmp/otkati-bind'
```

Save, then run:

```sh
hyprctl reload
```

Alt+Z saves a clip to `~/Videos/Откаты`. The name looks like `otkat1-2026-10-26.mp4`. Focus stays in the game.

The live ring records H.264 at 60 fps, about 10 minutes. Length, bitrate, and encoder are in the window, but they still do not change that recording.

## Installing again

The `~/otkati` folder is already there, so skip the clone:

```sh
"$HOME/otkati/install"
```
