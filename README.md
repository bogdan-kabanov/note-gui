# Записная книжка

Локальное десктоп-приложение для markdown-заметок. Файлы лежат в выбранной папке, служебные данные — в `.note-gui` внутри неё.

## Запуск

Нужны Node.js, Rust и Visual Studio Build Tools с компилятором C++.

```bash
npm install
npm run tauri dev
```

Версия клиента: `0.1.0` в `package.json`, `src/app_version.ts` и `src-tauri/tauri.conf.json`.

## Обновления

В настройках укажите адрес манифеста:

`https://gitlab.com/<group>/<project>/-/releases/permalink/latest/downloads/latest.json`

Пока адрес пуст, проверка сообщает, что источник обновлений не задан.

Релиз собирается в GitLab CI по тегу `vX.Y.Z`, совпадающему с версией приложения. Секреты runner:

- `TAURI_SIGNING_PRIVATE_KEY` — содержимое приватного ключа подписи
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — пароль ключа
- `GITLAB_TOKEN` — если job token не может создавать релизы

Публичный ключ уже записан в `src-tauri/tauri.conf.json`. Приватный ключ в git не хранится.

## Дальше

Сейчас сессия локальная. Контракт `VaultStore` рассчитан на позже подключаемый удалённый доступ кооперативного аккаунта; форма входа не показывается, пока API нет.
