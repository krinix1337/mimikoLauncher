<div align="center">

<a href="https://github.com/krinix1337/mimikoLauncher/releases/latest">
	<img src=".github/assets/hero.png" alt="mimikoLauncher — лаунчер Minecraft" width="100%" />
</a>

<br />
<br />

[![Релиз](https://img.shields.io/badge/v2.1-D0BCFF?style=for-the-badge&label=%D1%80%D0%B5%D0%BB%D0%B8%D0%B7&labelColor=211F26)](https://github.com/krinix1337/mimikoLauncher/releases/latest) [![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-D0BCFF?style=for-the-badge&logo=windows&logoColor=D0BCFF&labelColor=211F26)](https://github.com/krinix1337/mimikoLauncher/releases/latest) [![Tauri](https://img.shields.io/badge/Tauri-2-EFB8C8?style=for-the-badge&logo=tauri&logoColor=EFB8C8&labelColor=211F26)](https://tauri.app) [![Rust](https://img.shields.io/badge/Rust-1.95-EFB8C8?style=for-the-badge&logo=rust&logoColor=EFB8C8&labelColor=211F26)](https://www.rust-lang.org) [![Vue](https://img.shields.io/badge/Vue-3-CCC2DC?style=for-the-badge&logo=vuedotjs&logoColor=CCC2DC&labelColor=211F26)](https://vuejs.org) [![Лицензия](https://img.shields.io/badge/GPL--3.0-CCC2DC?style=for-the-badge&label=%D0%BB%D0%B8%D1%86%D0%B5%D0%BD%D0%B7%D0%B8%D1%8F&labelColor=211F26)](apps/app/LICENSE)

**[Скачать](https://github.com/krinix1337/mimikoLauncher/releases/latest)** &nbsp;·&nbsp; **[Возможности](#-возможности)** &nbsp;·&nbsp; **[Скриншоты](#-скриншоты)** &nbsp;·&nbsp; **[Установка](#-установка)** &nbsp;·&nbsp; **[mimiko.fun](https://mimiko.fun)**

</div>

<br />

## ✨ Возможности

<img src=".github/assets/features.png" alt="Возможности mimikoLauncher" width="100%" />

<details>
<summary><b>Подробнее о каждой возможности</b></summary>
<br />

| | Что умеет |
| :-- | :-- |
| 🎮 **Вход** | По нику без лицензии (UUID совпадает с офлайн-режимом сервера), через [Ely.by](https://ely.by) со скинами в игре или через аккаунт Microsoft |
| 🖼️ **Фон** | Восемь обоев в высоком качестве, своя картинка с компьютера и панорамы главного меню Minecraft — от Classic до 26.x, плюс первоапрельские. Панорама медленно вращается, как в игре |
| 🎨 **Оформление** | Material 3 Expressive, цветовые схемы, акцент под выбранные обои и матовое стекло рамки с настраиваемым размытием |
| 📦 **Контент** | Моды, шейдеры, ресурспаки и модпаки `.mrpack` с Modrinth. Fabric, Quilt, Forge и NeoForge |
| 📂 **Перенос** | Сборки из Prism Launcher, MultiMC и Modrinth App — достаточно выбрать папку, название и иконка сохранятся |
| ⚙️ **Настройки игры** | Загрузи готовый `options.txt` — графика, бинды и сервера будут одинаковыми во всех сборках |
| 🚫 **Без лишнего** | Никакой рекламы, баннеров и обязательного аккаунта |

</details>

## 📸 Скриншоты

<div align="center">

<img src=".github/assets/home.png" alt="Главная страница" width="100%" />

<sub><b>Главная</b> — недавние миры и библиотека сборок на обоях</sub>

<br />
<br />

<table>
<tr>
<td width="50%"><img src=".github/assets/appearance.png" alt="Обои и панорамы" /></td>
<td width="50%"><img src=".github/assets/browse.png" alt="Каталог модов" /></td>
</tr>
<tr>
<td align="center"><sub><b>Внешний вид</b> — обои и панорамы Minecraft</sub></td>
<td align="center"><sub><b>Каталог</b> — моды с Modrinth</sub></td>
</tr>
</table>

</div>

## ⬇️ Установка

<a href="https://github.com/krinix1337/mimikoLauncher/releases/latest">
	<img src=".github/assets/download.png" alt="Скачать mimikoLauncher" width="100%" />
</a>

1. Открой [последний релиз](https://github.com/krinix1337/mimikoLauncher/releases/latest) и скачай `mimikoLauncher_x.x.x_x64-setup.exe`.
2. Запусти установщик — права администратора не нужны, лаунчер ставится для текущего пользователя.
3. Готово: ярлык появится на рабочем столе и в меню «Пуск».

> [!WARNING]
> Установщик пока не подписан сертификатом. Если Windows покажет «Windows защитила ваш компьютер», нажми **Подробнее → Выполнить в любом случае**.

> [!TIP]
> Уже есть сборки в Prism Launcher или Modrinth App? Нажми **＋ → Импорт** и выбери папку сборки — всё перенесётся вместе с названием и иконкой.

<details>
<summary><h2>🛠️ Сборка из исходников</h2></summary>

Нужны **Rust** (`rustup`), **Node.js 24+**, **pnpm** и на Windows — **Visual Studio Build Tools** (C++).

```bash
pnpm install
cp packages/app-lib/.env.prod packages/app-lib/.env
pnpm app:dev      # запуск в режиме разработки
pnpm app:build    # сборка установщика
```

> [!NOTE]
> Первая сборка Rust долгая. Чтобы компьютер не лагал, ограничь потоки: `CARGO_BUILD_JOBS=4`.

</details>

## 📄 Лицензия

mimikoLauncher основан на открытом [Modrinth App](https://github.com/modrinth/code) и распространяется по лицензии **GPL-3.0** — см. [`apps/app/LICENSE`](apps/app/LICENSE).

<sub>Название и логотип Modrinth — товарные знаки Rinth, Inc.; проект с ними не связан. Обои — с [Unsplash](https://unsplash.com), панорамы — из Minecraft © Mojang Studios.</sub>
