#!/bin/bash

# Скрипт для проверки соответствия версии тега и версии в Cargo.toml
# Запускается как pre-commit хук при создании тега

set -e

# Получаем версию из Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Проверяем, есть ли тег в коммите
if git describe --exact-match --tags HEAD 2>/dev/null; then
    # Получаем версию из тега
    TAG_VERSION=$(git describe --exact-match --tags HEAD | sed 's/^v//')
    
    echo "Версия в Cargo.toml: $CARGO_VERSION"
    echo "Версия в теге: $TAG_VERSION"
    
    if [ "$CARGO_VERSION" = "$TAG_VERSION" ]; then
        echo "✅ Версии совпадают!"
    else
        echo "❌ Версии не совпадают!"
        echo "Версия в Cargo.toml: $CARGO_VERSION"
        echo "Версия в теге: $TAG_VERSION"
        echo ""
        echo "Для исправления:"
        echo "1. Обновите версию в Cargo.toml до $TAG_VERSION"
        echo "2. Или создайте тег с версией $CARGO_VERSION"
        exit 1
    fi
else
    # Если это не тег, просто проверяем что версия в Cargo.toml корректная
    echo "Версия в Cargo.toml: $CARGO_VERSION"
    echo "ℹ️  Это не тег, проверка пропущена"
fi 