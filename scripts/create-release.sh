#!/bin/bash

# Скрипт для создания релиза с строгой проверкой версий

set -e

if [ $# -ne 1 ]; then
    echo "Использование: $0 <версия>"
    echo "Пример: $0 0.1.3"
    exit 1
fi

VERSION=$1

echo "Создание релиза версии $VERSION..."

# Проверяем, что версия в Cargo.toml соответствует
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ "$CARGO_VERSION" != "$VERSION" ]; then
    echo "❌ ОШИБКА: Версия в Cargo.toml ($CARGO_VERSION) не соответствует запрошенной версии ($VERSION)"
    echo ""
    echo "Для исправления:"
    echo "1. Обновите версию в Cargo.toml вручную:"
    echo "   sed -i 's/^version = \".*\"/version = \"$VERSION\"/' Cargo.toml"
    echo ""
    echo "2. Зафиксируйте изменения:"
    echo "   git add Cargo.toml && git commit -m 'Bump version to $VERSION'"
    echo ""
    echo "3. Затем запустите этот скрипт снова"
    exit 1
fi

# Проверяем, что нет незафиксированных изменений
if ! git diff-index --quiet HEAD --; then
    echo "❌ Есть незафиксированные изменения. Сначала зафиксируйте их:"
    echo "   git add . && git commit -m 'Prepare release $VERSION'"
    exit 1
fi

# Создаем тег
echo "Создаю тег v$VERSION..."
git tag v$VERSION

echo "✅ v$VERSION создан!"
echo ""
echo "Следующие шаги:"
echo "1. Отправьте тег: git push origin v$VERSION"
echo "2. CI автоматически создаст релиз на GitHub"
echo ""
echo "Или выполните все сразу:"
echo "   git push origin v$VERSION"
