<div align="center">

<img src="apps/app/icons/128x128@2x.png" width="128" alt="mimikoLauncher" />

# mimikoLauncher

**Красивый лаунчер Minecraft на основе Modrinth App: без рекламы, без обязательного аккаунта Microsoft, с обоями, панорамами и Material 3.**

[![Скачать](https://img.shields.io/github/v/release/krinix1337/mimikoLauncher?style=for-the-badge&label=%D0%A1%D0%BA%D0%B0%D1%87%D0%B0%D1%82%D1%8C&color=7c5cc4&logo=windows&logoColor=white)](https://github.com/krinix1337/mimikoLauncher/releases/latest)
[![Tauri](https://img.shields.io/badge/Tauri-2-7c5cc4?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.95-7c5cc4?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-7c5cc4?style=for-the-badge&logo=vuedotjs&logoColor=white)](https://vuejs.org)
[![License](https://img.shields.io/badge/license-GPL--3.0-7c5cc4?style=for-the-badge)](apps/app/LICENSE)

[**⬇️ Скачать для Windows**](https://github.com/krinix1337/mimikoLauncher/releases/latest) · [mimiko.fun](https://mimiko.fun)

<br />

<img src=".github/assets/home.png" alt="Главная страница mimikoLauncher" width="900" />

</div>

---

## ✨ Возможности

<table>
<tr>
<td width="50%" valign="top">

### 🎮 Играй как хочешь
- **Вход по нику** — без лицензии: ввёл ник и в игру. UUID совпадает с офлайн-режимом сервера
- **Ely.by** — вход через аккаунт [Ely.by](https://ely.by), скины в игре через authlib-injector
- **Аккаунт Microsoft** — тоже работает, если есть лицензия

</td>
<td width="50%" valign="top">

### 🖼️ Обои и панорамы
- **Подборка обоев** в высоком качестве и **своя картинка** с компьютера
- **Панорамы главного меню Minecraft** всех версий, от Classic до 26.x, плюс первоапрельские
- **Акцент под фон** — цвет интерфейса сам подстраивается под выбранные обои

</td>
</tr>
<tr>
<td width="50%" valign="top">

### 🎨 Material 3 и стекло
- Оформление **Material 3 Expressive** со своими цветовыми схемами
- **Матовое стекло** рамки и панелей с настраиваемой силой размытия
- Без рекламы, баннеров и окон согласия

</td>
<td width="50%" valign="top">

### 📦 Сборки и моды
- Поиск и установка модов, шейдеров и ресурспаков с **Modrinth**, `.mrpack`
- Fabric / Quilt / Forge / NeoForge
- **Перенос сборок** из Prism Launcher, MultiMC и Modrinth App — просто выбери папку, название и иконка сохранятся
- **Общие настройки игры** — загрузи готовый `options.txt`, и графика, бинды и сервера будут во всех сборках

</td>
</tr>
</table>

## 📸 Скриншоты

<div align="center">

<img src=".github/assets/appearance.png" alt="Выбор обоев и панорам" width="900" />

<sub>Обои и панорамы Minecraft в настройках внешнего вида</sub>

<br /><br />

<img src=".github/assets/browse.png" alt="Каталог модов" width="900" />

<sub>Каталог модов Modrinth</sub>

</div>

## ⬇️ Установка

1. Скачай `mimikoLauncher_x.x.x_x64-setup.exe` со страницы [**релизов**](https://github.com/krinix1337/mimikoLauncher/releases/latest).
2. Запусти установщик. Права администратора не нужны, лаунчер ставится только для текущего пользователя.
3. Если Windows покажет «Windows защитила ваш компьютер», нажми **Подробнее → Выполнить в любом случае**: установщик пока не подписан сертификатом.

## 🛠️ Сборка из исходников

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
- [x] Обои, панорамы Minecraft и акцент под фон
- [x] Material 3 и матовое стекло
- [x] Перенос сборок из других лаунчеров и общий `options.txt`
- [ ] Скины Ely.by в превью лаунчера
- [ ] Автообновление лаунчера

## 📄 Лицензия

mimikoLauncher основан на открытом [Modrinth App](https://github.com/modrinth/code) и распространяется по лицензии **GPL-3.0** (см. [`apps/app/LICENSE`](apps/app/LICENSE)).
Название и логотип Modrinth — товарные знаки Rinth, Inc.; проект с ними не связан и их не использует.
Обои — с [Unsplash](https://unsplash.com), панорамы — из игры Minecraft © Mojang Studios.
