# Обнаружение документации QSL в поиске

## Реализовано

Текущие страницы имеют уникальные заголовки и описания, canonical, взаимные RU/EN hreflang, английскую главную en.html с готовым HTML, JSON-LD SoftwareSourceCode/WebPage, Open Graph/Twitter и превью 1200×630. Sitemap содержит только актуальные canonical-страницы. Копии старых отчётов и автономный дубликат имеют noindex; сохранённые архивы не добавляются в карту сайта. Рейтинги, отзывы, ускорение и позиции в поиске не выдумываются.

Ссылки сайта на исходники ведут к тегу GitHub, а не к недоступным путям вне корня Pages. Старые отчёты опубликованы отдельными копиями; оригинальные артефакты не изменяются. Пересборка: `python reference/build-site.py .`, затем `python reference/verify-site.py` и `python reference/verify-seo.py`.

## GitHub Pages и robots.txt

Проект расположен по адресу https://zadr0.github.io/QSL/. Роботы читают https://zadr0.github.io/robots.txt, а не /QSL/robots.txt. Корневой файл сейчас возвращает 404; запрета обхода не обнаружено. Подмена его неработающим файлом в каталоге проекта не используется. `reference/robots-root.example.txt` — пример для владельца корня хоста или будущего собственного домена; это не опубликованное правило корня. [Требования Google к расположению robots](https://developers.google.com/crawling/docs/robots-txt/create-robots-txt).

Карта сайта: https://zadr0.github.io/QSL/sitemap.xml; ссылка присутствует в HTML. Отправка через Search Console требует подтверждённого ресурса и не объявляется выполненной без доступа к аккаунту. IndexNow использует keyLocation внутри проекта; отправка выполняется только после публичного развёртывания ключа и страниц. Приём запроса не означает индексацию или гарантированные позиции. [Протокол IndexNow](https://www.indexnow.org/documentation).

## Границы результата

Описание и темы GitHub, версионный Release, ссылки и категории crates.io связывают одну идентичность проекта. Поисковые системы самостоятельно выбирают момент обхода, индексации и ранжирование. Техническое SEO не доказывает спрос и не гарантирует верхние позиции. [Sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview), [локализации](https://developers.google.com/search/docs/specialty/international/localized-versions), [SoftwareSourceCode](https://schema.org/SoftwareSourceCode).
