# Scrappeur Wikipedia en Rust

Un scrappeur web performant et interactif développé en Rust pour extraire automatiquement des informations depuis les pages Wikipedia.

## 📋 Table des matières

- [Description](#description)
- [Fonctionnalités](#fonctionnalités)
- [Structure du projet](#structure-du-projet)
- [Dépendances](#dépendances)
- [Installation](#installation)
- [Utilisation](#utilisation)
- [Résolution des problèmes](#résolution-des-problèmes)
- [Exemples](#exemples)

## 🎯 Description

Ce projet permet de scraper des pages Wikipedia en français et d'extraire :
- Le titre de la page
- Le résumé (premier paragraphe)
- Les sections et sous-sections
- Les liens internes vers d'autres pages
- Les images présentes sur la page

Les données extraites sont ensuite sauvegardées au format JSON avec le nom du sujet recherché.

## ✨ Fonctionnalités

- ✅ **Mode interactif** : interface utilisateur en ligne de commande
- ✅ **Arguments CLI** : utilisation via arguments (--sujet, --url, --interactif)
- ✅ **Recherche flexible** : par sujet, URL complète ou mot-clé
- ✅ Scraping asynchrone pour de meilleures performances
- ✅ Extraction structurée des données
- ✅ Export automatique en JSON avec nom personnalisé
- ✅ Gestion des erreurs robuste
- ✅ User-Agent personnalisé pour éviter les blocages
- ✅ Filtrage intelligent des liens et images

## 📁 Structure du projet

```
Scrappeur wikipedia/
│
├── src/
│   └── main.rs              # Fichier principal contenant toute la logique
│
├── target/                  # Dossier généré par Cargo (compilations)
│   ├── debug/              # Build en mode debug
│   └── release/            # Build en mode release (optimisé)
│
├── Cargo.toml              # Manifeste du projet et dépendances
├── Cargo.lock              # Verrouillage des versions des dépendances
├── .gitignore              # Fichiers à ignorer par Git
├── README.md               # Ce fichier
└── *.json                  # Fichiers JSON générés après exécution
```

### Description des fichiers et dossiers

#### `src/main.rs`
**Rôle** : Fichier source principal du projet
- **Structure `WikipediaPage`** : Stocke les données extraites
- **Structure `Args`** : Gère les arguments CLI avec clap
- **Fonction `main()`** : Point d'entrée asynchrone, gère la logique de choix (interactif/CLI)
- **Fonction `get_user_input()`** : Interface interactive pour choisir le mode de recherche
- **Fonction `sanitize_filename()`** : Nettoie le nom du fichier JSON généré
- **Fonction `scrape_wikipedia()`** : Logique de scraping et parsing HTML avec sélecteurs CSS

#### `Cargo.toml`
**Rôle** : Fichier de configuration du projet
- Définit le nom, version et édition Rust du projet
- Liste toutes les dépendances externes nécessaires
- Configure les features spécifiques des dépendances

#### `Cargo.lock`
**Rôle** : Verrouillage des versions
- Généré automatiquement par Cargo
- Garantit la reproductibilité des builds
- **Ne pas modifier manuellement**

#### `target/`
**Rôle** : Dossier de compilation
- Généré automatiquement lors du `cargo build` ou `cargo run`
- Contient les fichiers binaires compilés et les dépendances
- Peut être supprimé sans risque (`cargo clean`)

#### `.gitignore`
**Rôle** : Exclusions Git
- Empêche le versionnement de `target/`, fichiers JSON et `Cargo.lock`
- Réduit la taille du repository

#### `*.json` (ex: `Rust_(langage).json`)
**Rôle** : Sorties du scraping
- Générés après l'exécution du programme
- Nom basé sur le titre de la page Wikipedia
- Contiennent les données extraites au format JSON structuré

## 📦 Dépendances

Le projet utilise les crates suivantes :

### 1. **reqwest** (v0.11)
- **Rôle** : Client HTTP asynchrone
- **Usage** : Récupération du contenu HTML des pages Wikipedia
- **Feature** : `json` pour la sérialisation JSON

### 2. **tokio** (v1.0)
- **Rôle** : Runtime asynchrone
- **Usage** : Permet l'exécution asynchrone du code
- **Feature** : `full` pour toutes les fonctionnalités

### 3. **scraper** (v0.18)
- **Rôle** : Parser et sélecteur HTML
- **Usage** : Extraction des éléments HTML via sélecteurs CSS
- **Basé sur** : html5ever et selectors

### 4. **serde** (v1.0)
- **Rôle** : Framework de sérialisation/désérialisation
- **Usage** : Conversion des structures Rust en JSON
- **Feature** : `derive` pour les macros

### 5. **serde_json** (v1.0)
- **Rôle** : Support JSON pour serde
- **Usage** : Écriture du fichier JSON de sortie

### 6. **clap** (v4.5) ⭐ NOUVEAU
- **Rôle** : Parser d'arguments en ligne de commande
- **Usage** : Gestion des options --sujet, --url, --interactif
- **Feature** : `derive` pour les macros

### Graphe de dépendances

```
main.rs
  ├── reqwest (HTTP client)
  │     └── tokio (runtime async)
  ├── scraper (HTML parsing)
  │     ├── html5ever
  │     └── selectors
  ├── serde (serialization)
  ├── serde_json (JSON support)
  │     └── serde
  └── clap (CLI arguments) ⭐ NOUVEAU
```

## 🚀 Installation

### Prérequis

- **Rust** : Version 1.70 ou supérieure
- **Cargo** : Gestionnaire de paquets Rust (inclus avec Rust)

### Installer Rust

Si vous n'avez pas Rust installé :

**Windows** :
```powershell
# Téléchargez et exécutez rustup-init.exe depuis https://rustup.rs/
# Ou utilisez winget :
winget install Rustlang.Rustup
```

**Linux/macOS** :
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Cloner ou créer le projet

```bash
# Si vous clonez depuis Git
git clone <votre-repo>
cd "Scrappeur wikipedia"

# Ou créez un nouveau projet
cargo new "Scrappeur wikipedia"
cd "Scrappeur wikipedia"
```

### Installer les dépendances

```bash
# Les dépendances seront automatiquement téléchargées lors du premier build
cargo build
```

## 💻 Utilisation

### 🎮 Mode 1 : Interactif (Recommandé)

Le mode par défaut qui vous guide pas à pas :

```bash
cargo run
```

Vous verrez un menu :
```
=== Scraper Wikipedia ===

Choisissez une option :
1. Entrer un sujet (ex: Rust_(langage))
2. Entrer une URL complète
3. Rechercher par mot-clé

Votre choix (1-3) :
```

**Option 1** : Format exact de Wikipedia (avec underscores)
- Exemple : `Python_(langage)`, `Intelligence_artificielle`

**Option 2** : Coller l'URL complète
- Exemple : `https://fr.wikipedia.org/wiki/JavaScript`

**Option 3** : Recherche simple (espaces automatiquement convertis)
- Exemple : `langage de programmation` → `langage_de_programmation`

### 🚀 Mode 2 : Ligne de commande

#### Recherche par sujet
```bash
cargo run -- --sujet "Rust_(langage)"
cargo run -- -s "Python_(langage)"
```

#### Recherche par URL complète
```bash
cargo run -- --url "https://fr.wikipedia.org/wiki/JavaScript"
cargo run -- -u "https://fr.wikipedia.org/wiki/TypeScript"
```

#### Forcer le mode interactif
```bash
cargo run -- --interactif
cargo run -- -i
```

### 📖 Aide

Afficher toutes les options disponibles :
```bash
cargo run -- --help
```

### 📤 Sortie du programme

Le programme :
1. **Affiche dans le terminal** :
   - Le titre de la page
   - Le résumé complet
   - Le nombre de sections, liens et images
   - Les 5 premières sections

2. **Génère un fichier JSON** :
   - Nom : `{Titre_de_la_page}.json`
   - Contenu : Toutes les données structurées

### Exemple de sortie console

```
Scraping de la page Wikipedia : https://fr.wikipedia.org/wiki/Rust_(langage)

=== Résultats du Scraping ===

Titre: Rust (langage)

Résumé:
Rust est un langage de programmation compilé multi-paradigme...

Nombre de sections: 15
Nombre de liens: 50
Nombre d'images: 20

Premières sections:
  1. Histoire
  2. Caractéristiques
  3. Syntaxe
  4. Gestion de la mémoire
  5. Écosystème

Données sauvegardées dans 'Rust_(langage).json'
```

### Exemple de fichier JSON généré

```json
{
  "title": "Rust (langage)",
  "summary": "Rust est un langage de programmation compilé multi-paradigme...",
  "sections": [
    "Histoire",
    "Caractéristiques",
    "Syntaxe",
    "Gestion de la mémoire",
    "Écosystème"
  ],
  "links": [
    "https://fr.wikipedia.org/wiki/Langage_de_programmation",
    "https://fr.wikipedia.org/wiki/Mozilla",
    "https://fr.wikipedia.org/wiki/C%2B%2B"
  ],
  "images": [
    "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/1200px-Rust_programming_language_black_logo.svg.png"
  ]
}
```

## 🔧 Résolution des problèmes

### Erreur "Accès refusé" (Windows)

**Problème** : `error: Accès refusé. (os error 5)`

**Solutions** :
1. **Fermez tous les IDE/éditeurs** ouverts sur le projet
2. **Nettoyez le projet** : `cargo clean`
3. **Exécutez PowerShell en tant qu'administrateur`
4. **⚠️ IMPORTANT : Déplacez le projet hors de OneDrive** 
   - De : `C:\Users\Admin\OneDrive\Bureau\ESGI\BAC +4\RUST\Scrappeur wikipedia`
   - Vers : `C:\Users\Admin\Documents\RUST\Scrappeur wikipedia`
   - OneDrive interfère avec la compilation Rust
5. **Ajoutez une exclusion antivirus** pour le dossier `target/`

### Erreur de connexion réseau

**Problème** : Le scraping échoue avec une erreur réseau

**Solutions** :
- Vérifiez votre connexion Internet
- Vérifiez que Wikipedia n'est pas bloqué par votre pare-feu
- Le serveur peut avoir temporairement bloqué les requêtes : attendez quelques minutes
- Essayez avec un autre sujet

### Le JSON est vide ou incomplet

**Problème** : Les sélecteurs ne trouvent pas les éléments

**Solutions** :
- Wikipedia peut avoir changé sa structure HTML
- Vérifiez que l'URL est correcte et la page existe
- Certaines pages Wikipedia ont une structure différente
- Testez avec des pages populaires : `Rust_(langage)`, `Python_(langage)`, `JavaScript`

### Page Wikipedia introuvable

**Problème** : Erreur 404 ou page vide

**Solutions** :
- Vérifiez l'orthographe du sujet
- Utilisez le format exact de Wikipedia (avec underscores et parenthèses)
- Copiez l'URL directement depuis votre navigateur (option 2 du mode interactif)
- Essayez la recherche par mot-clé (option 3)

### Caractères spéciaux dans le nom de fichier

**Problème** : Le fichier JSON n'est pas créé

**Solutions** :
- La fonction `sanitize_filename()` remplace automatiquement les caractères interdits
- Si le problème persiste, le titre sera tronqué à 50 caractères maximum

## 📚 Exemples d'utilisation

### Exemple 1 : Scraper plusieurs sujets en boucle

Créez un script bash/PowerShell :

**PowerShell** :
```powershell
$sujets = @(
    "Rust_(langage)",
    "Python_(langage)",
    "JavaScript",
    "Intelligence_artificielle"
)

foreach ($sujet in $sujets) {
    Write-Host "Scraping de $sujet..." -ForegroundColor Green
    cargo run -- --sujet $sujet
    Start-Sleep -Seconds 2  # Pause de 2 secondes entre chaque requête
}
```

**Bash** :
```bash
#!/bin/bash
sujets=("Rust_(langage)" "Python_(langage)" "JavaScript" "Intelligence_artificielle")

for sujet in "${sujets[@]}"; do
    echo "Scraping de $sujet..."
    cargo run -- --sujet "$sujet"
    sleep 2  # Pause de 2 secondes
done
```

### Exemple 2 : Modifier le code pour extraire plus d'informations

Pour limiter ou augmenter le nombre de liens/images, modifiez dans `src/main.rs` :

```rust
// Actuellement : 50 liens maximum
.take(50)  // Changez en .take(100) pour plus de liens

// Actuellement : 20 images maximum
.take(20)  // Changez en .take(50) pour plus d'images
```

### Exemple 3 : Filtrer les sections spécifiques

Après le scraping, ajoutez dans `main()` :

```rust
// Extraire seulement les sections contenant certains mots-clés
let sections_filtrees: Vec<String> = page_data.sections
    .iter()
    .filter(|s| s.contains("Histoire") || s.contains("Syntaxe"))
    .cloned()
    .collect();

println!("Sections filtrées: {:?}", sections_filtrees);
```

### Exemple 4 : Scraper en mode release (plus rapide)

```bash
cargo build --release
./target/release/wikipedia_scraper --sujet "Rust_(langage)"
```

## 🎓 Cas d'usage pédagogiques

Ce projet est idéal pour apprendre :
- **Web scraping** avec Rust
- **Programmation asynchrone** avec tokio
- **Parsing HTML** avec scraper
- **CLI arguments** avec clap
- **Sérialisation JSON** avec serde
- **Gestion d'erreurs** en Rust
- **Requêtes HTTP** avec reqwest

## 📝 Licence

Ce projet est à usage éducatif. Respectez les conditions d'utilisation de Wikipedia lors du scraping :
- Ne pas surcharger les serveurs (ajoutez des pauses entre les requêtes)
- Respecter le fichier robots.txt
- Utiliser pour des fins d'apprentissage

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
- Signaler des bugs via les issues
- Proposer des améliorations
- Ajouter de nouvelles fonctionnalités (export CSV, scraping multilingue, etc.)
- Améliorer la documentation

## 📞 Contact

Projet ESGI - BAC +4 RUST

---

**Note importante** : Ce scrappeur utilise un User-Agent approprié et respecte les bonnes pratiques du web scraping. Utilisez-le de manière responsable et éthique.
