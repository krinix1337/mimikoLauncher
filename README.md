<div align="center">

<img src="apps/app/icons/128x128@2x.png" width="128" alt="mimikoLauncher" />

# mimikoLauncher

**Лаунчер Minecraft с дизайном Modrinth App — без рекламы и без обязательного аккаунта Microsoft.**

[![Tauri](https://img.shields.io/badge/Tauri-2-1bd96a?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.95-1bd96a?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-1bd96a?style=for-the-badge&logo=vuedotjs&logoColor=white)](https://vuejs.org)
[![License](https://img.shields.io/badge/license-GPL--3.0-1bd96a?style=for-the-badge)](apps/app/LICENSE)

[mimiko.fun](https://mimiko.fun)

</div>

---

## ✨ Возможности

| | |
|---|---|
| 🎮 **Вход по нику** | Играй без лицензии: введи ник — и в игру. UUID совпадает с офлайн-режимом сервера |
| 🧑‍🎨 **Ely.by** | Вход через аккаунт [Ely.by](https://ely.by): скины в игре через authlib-injector |
| 🚫 **Без рекламы** | Никаких баннеров, промо и окон согласия на рекламу |
| 📦 **Моды и модпаки** | Поиск и установка с Modrinth, `.mrpack`, Fabric / Quilt / Forge / NeoForge |
| 🗂️ **Сборки** | Отдельные экземпляры со своей версией, модами и настройками Java |
| ⚡ **Лёгкий** | Tauri вместо Electron — меньше памяти, больше ОЗУ для игры |

## 🛠️ Сборка

Нужны **Rust** (`rustup`), **Node.js 24+**, **pnpm** и на Windows — **Visual Studio Build Tools** (C++).

```bash
pnpm install
cp packages/app-lib/.env.prod packages/app-lib/.env
pnpm app:dev      # запуск в режиме разработки
pnpm app:build    # сборка установщика
```

> Первая сборка Rust долгая. Чтобы компьютер не лагал, ограничь потоки: `CARGO_BUILD_JOBS=4`.

## 🗺️ Планы

- [x] Убрать рекламу и сервисы Modrinth (друзья, новости, Modrinth+)
- [x] Вход по нику без Microsoft
- [x] Вход через Ely.by и скины (authlib-injector)
- [x] Вырезать хостинг, аккаунт Modrinth и общие сборки
- [ ] Скины Ely.by в превью лаунчера
- [ ] Свои фишки mimiko

## 📄 Лицензия

mimikoLauncher основан на открытом [Modrinth App](https://github.com/modrinth/code) и распространяется по лицензии **GPL-3.0** (см. [`apps/app/LICENSE`](apps/app/LICENSE)).
Название и логотип Modrinth — товарные знаки Rinth, Inc.; проект с ними не связан и их не использует.
