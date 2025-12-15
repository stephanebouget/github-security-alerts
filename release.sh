#!/bin/bash

# Script de release automatique
# Usage: ./release.sh 1.2.3

if [ $# -eq 0 ]; then
    echo "❌ Usage: $0 <version>"
    echo "   Exemple: $0 1.2.3"
    exit 1
fi

NEW_VERSION="$1"

# Détecter automatiquement la version actuelle depuis package.json
CURRENT_VERSION=$(grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')

echo "🚀 Release automatique: $CURRENT_VERSION → $NEW_VERSION"

# Vérifier que nous sommes dans le bon répertoire
if [ ! -f "package.json" ] || [ ! -f "src-tauri/Cargo.toml" ]; then
    echo "❌ Erreur: Exécutez le script depuis la racine du projet"
    exit 1
fi

echo "📝 Mise à jour des versions dans tous les fichiers..."

# 1. package.json
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/g" package.json

# 2. src-tauri/Cargo.toml
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/g" src-tauri/Cargo.toml

# 3. src-tauri/tauri.conf.json
if [ -f "src-tauri/tauri.conf.json" ]; then
    sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/g" src-tauri/tauri.conf.json
fi

# 4. src-tauri/tauri.conf.json.example  
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/g" src-tauri/tauri.conf.json.example

# 5. Footer component
sed -i "s/>v$CURRENT_VERSION</>v$NEW_VERSION</g" src/app/shared/components/footer/footer.component.html

# 6. setup.sh
sed -i "s/git tag v$CURRENT_VERSION/git tag v$NEW_VERSION/g" setup.sh

# 7. Mettre à jour Cargo.lock
echo "🔧 Mise à jour de Cargo.lock..."
cd src-tauri
cargo update
cd ..

echo "✅ Toutes les versions ont été mises à jour"

# Vérifier les changements
echo "📋 Fichiers modifiés:"
git diff --name-only

echo "🤔 Voulez-vous continuer avec le commit et le tag ? (y/N)"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    # Commit
    echo "💾 Commit des modifications..."
    git add .
    git commit -m "chore: bump version to $NEW_VERSION"
    
    # Tag
    echo "🏷️ Création du tag v$NEW_VERSION..."
    git tag "v$NEW_VERSION"
    
    echo "🎉 Release $NEW_VERSION prête !"
    echo ""
    echo "📤 Pour publier:"
    echo "   git push origin main --tags"
    echo ""
    echo "🤖 GitHub Actions va automatiquement:"
    echo "   • Compiler l'app"
    echo "   • Créer la release"
    echo "   • Générer les mises à jour OTA"
else
    echo "❌ Annulé. Vous pouvez annuler les modifications avec: git checkout ."
fi