# 19 — Жизнь в сессии

**What to build:** Кольцо стартует вместе с сессией Hyprland, так что перед игрой его не нужно открывать. Перезапуск оболочки не завершает процесс кольца и не выбрасывает ещё не сохранённые минуты.

**Blocked by:** 04 — Живое кольцо

**Status:** resolved

- [x] После входа в сессию Hyprland кольцо уже запущено
- [x] Перезапуск Serpantinum не завершает процесс кольца
- [x] Минуты, которые кольцо уже держит, после перезапуска оболочки всё ещё сохраняются биндом

## Comments

Resolved on `otkati` by merge `98625435dc249278c0ea9107e775f644d78cc4f9`. The ring starts from `systemd/user/otkati-ring.service`, not as a child of the shell.
