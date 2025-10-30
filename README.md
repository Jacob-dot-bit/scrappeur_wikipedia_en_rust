# Scrappeur Wikipedia en Rust 🦀

Un scrappeur web performant et interactif développé en Rust pour extraire automatiquement des informations depuis les pages Wikipedia.

## 📋 Table des matières

- [Description](#description)
- [Fonctionnalités](#fonctionnalités)
- [Architecture technique](#architecture-technique)
- [Structure du projet](#structure-du-projet)
- [Dépendances](#dépendances)
- [Installation](#installation)
- [Utilisation](#utilisation)
- [Exemples](#exemples)
- [Résolution des problèmes](#résolution-des-problèmes)

## 🎯 Description

Ce projet permet de scraper des pages Wikipedia de deux façons :
1. **URLs directes** : Fournir une liste d'URLs Wikipedia à scraper
2. **Recherche par mot-clé** : Rechercher un sujet et scraper automatiquement les résultats

Les données extraites incluent :
- Le titre de la page
- Le résumé (premier paragraphe)
- Les sections et sous-sections
- Les liens internes vers d'autres pages
- Les images présentes sur la page

Toutes les données sont organisées dans des **dossiers par recherche** avec un résumé global et sauvegardées en plusieurs formats (JSON, Markdown, TXT).

## ✨ Fonctionnalités

- ✅ **Recherche par mot-clé** : Recherche automatique via l'API OpenSearch de Wikipedia
- ✅ **Nombre de résultats personnalisable** : Choisir combien d'articles scraper (1-20)
- ✅ **Organisation par recherche** : Un dossier timestampé par recherche avec tous les articles
- ✅ **Résumé global** : Fichier `RESUME_RECHERCHE.md` avec statistiques et liens
- ✅ **URLs directes** : Scraping d'URLs spécifiques
- ✅ **Mode interactif** : Interface CLI guidée avec choix du nombre de résultats
- ✅ **Arguments CLI** : Utilisation via ligne de commande avec clap
- ✅ **Multi-formats** : JSON, Markdown, TXT pour les différentes données
- ✅ **Filtrage intelligent** : Exclusion automatique des icônes et petites images
- ✅ **Support HTTPS** : Connexion sécurisée avec rustls
- ✅ **Requêtes HTTP manuelles** : Construction manuelle des requêtes HTTP/HTTPS
- ✅ **Gestion des erreurs robuste**
- ✅ **Pause entre requêtes** : Respectueux des serveurs Wikipedia

## 🏗️ Architecture technique

### Respect de la contrainte "Pas de lib pour les requêtes réseau"

**Notre approche :**

✅ **Requêtes HTTP construites manuellement**
- Utilisation directe de `std::net::TcpStream` (bibliothèque standard)
- Construction manuelle des headers HTTP (`GET`, `Host`, `User-Agent`, etc.)
- Parsing manuel des réponses HTTP (séparation headers/body)
- **Aucune** bibliothèque HTTP de haut niveau (`reqwest`, `hyper`, `curl`, etc.)

⚠️ **Exception nécessaire : TLS/HTTPS**
- Wikipedia **force HTTPS** (redirection automatique HTTP → HTTPS)
- Implémentation TLS manuelle = plusieurs mois de travail + risques sécurité
- Solution : `rustls` = bibliothèque de **cryptographie**, pas de requêtes HTTP
- `rustls` fait uniquement le chiffrement TLS, nous gérons toujours HTTP manuellement

### Flux de données

```
Mot-clé utilisateur
    ↓
API OpenSearch Wikipedia (JSON)
    ↓
Liste d'URLs
    ↓
Requête HTTPS manuelle (TcpStream + rustls)
    ↓
HTML parsing (scraper)
    ↓
Extraction données
    ↓
Organisation en dossiers
    ↓
Sauvegarde fichiers (JSON + Markdown + TXT)
    ↓
Génération RESUME_RECHERCHE.md
```

## 📁 Structure du projet

```
Scrappeur wikipedia/
│
├── src/
│   └── main.rs              # Fichier principal (~700 lignes)
│       ├── Structures
│       │   ├── WikipediaPage      # Données extraites
│       │   └── Args               # Arguments CLI
│       ├── Fonctions principales
│       │   ├── main()
│       │   ├── rechercher_wikipedia()
│       │   ├── scrape_wikipedia()
│       │   └── generate_search_summary()
│       ├── Réseau HTTP/HTTPS
│       │   ├── http_get()
│       │   ├── https_get()
│       │   ├── extract_header()
│       │   └── parse_url()
│       ├── Extraction de contenu
│       │   └── extract_summary()
│       ├── Utilitaires
│       │   ├── url_encode()
│       │   └── get_urls_interactif()
│       └── Sauvegarde
│           ├── save_page_data()
│           └── generate_markdown()
│
├── resultats/               # Dossier généré après exécution
│   ├── Avion_20240116_143025/     # Dossier de recherche
│   │   ├── RESUME_RECHERCHE.md   # ← Résumé global de la recherche                
│   │   ├── avion.md    # Article 1
│   │   ├── Avion_de_ligne.md       # Article 2
│   │   └── Boeing_747.md          # Article 3
│   │       └── ...
│   └── BMW_20240116_150530/      # Autre recherche
│       ├── RESUME_RECHERCHE.md
│       └── ...
│
├── Cargo.toml              # Manifeste et dépendances
├── Cargo.lock              # Verrouillage des versions
├── .gitignore
└── README.md
```

## 📦 Dépendances

### Dépendances principales

| Crate | Version | Rôle |
|-------|---------|------|
| **scraper** | 0.18 | Parser HTML avec sélecteurs CSS |
| **serde** | 1.0 | Sérialisation des structures |
| **serde_json** | 1.0 | Export JSON |
| **clap** | 4.5 | Parser d'arguments CLI |
| **rustls** | 0.22 | Implémentation TLS pure Rust |
| **webpki-roots** | 0.26 | Certificats racines pour TLS |
| **chrono** | 0.4 | Gestion des dates (timestamps) |
| **sanitize-filename** | 0.5 | Nettoyage des noms de fichiers |

### Pourquoi rustls ?

- ✅ **Implémentation TLS pure Rust** (pas de dépendance OpenSSL)
- ✅ **Respecte la contrainte** : rustls est une lib de sécurité/cryptographie, pas de requêtes HTTP
- ✅ **Nous construisons toujours les requêtes HTTP manuellement**
- ✅ **Nécessaire** : Wikipedia force HTTPS

## 🚀 Installation

### Prérequis

- **Rust** : Version 1.70 ou supérieure
- **Cargo** : Gestionnaire de paquets Rust (inclus avec Rust)

### Installer Rust

**Windows** :
```powershell
winget install Rustlang.Rustup
```

**Linux/macOS** :
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Cloner le projet

```bash
git clone https://github.com/Jacob-dot-bit/scrappeur_wikipedia_en_rust/
cd scrappeur_wikipedia_en_rust
```

### Compiler le projet

```bash
# Mode debug (développement)
cargo build

# Mode release (optimisé)
cargo build --release
```

## 💻 Utilisation

### 🔍 Mode 1 : Recherche par mot-clé

**Ligne de commande** :
```bash
# Rechercher "avion" et scraper les 5 premiers résultats (par défaut)
cargo run -- -k "avion"

# Rechercher et personnaliser le nombre de résultats
cargo run -- -k "intelligence artificielle" -n 10

# Personnaliser le dossier de sortie
cargo run -- -k "Python" -n 5 --output mes_resultats
```

### 🔗 Mode 2 : URLs directes

**Avec un fichier** :
```bash
# Créer urls.txt avec vos URLs (une par ligne)
cargo run -- -f urls.txt
```

**Exemple de `urls.txt`** :
```
https://fr.wikipedia.org/wiki/Rust_(langage)
https://fr.wikipedia.org/wiki/Python_(langage)
https://fr.wikipedia.org/wiki/JavaScript
```

**En ligne de commande** :
```bash
# URLs séparées par des virgules
cargo run -- -u "https://fr.wikipedia.org/wiki/Rust_(langage),https://fr.wikipedia.org/wiki/Python_(langage)"
```

### 🎮 Mode 3 : Interactif

**Sans arguments** :
```bash
cargo run
```

Menu interactif :
```
=== Scraper Wikipedia (Mode interactif) ===

Choisissez une option :
1. Entrer des URLs directement
2. Rechercher par mot-clé

Votre choix (1-2) : 2
Entrez le mot-clé à rechercher : Avion
Nombre de résultats à scraper (défaut: 5, max 20) : 8
```

### 📖 Aide complète

```bash
cargo run -- --help
```

## 📚 Exemples

### Exemple 1 : Rechercher "Avion" avec 8 résultats

```bash
cargo run -- -k "Avion" -n 8
```

**Structure créée :**
```
resultats/
└── Avion_20240116_143025/
    ├── RESUME_RECHERCHE.md    ← Résumé global avec tableau et stats
    ├── Avion.md
    ├── Avion_de_ligne.md
    └── ... (7 autres articles)
```

**Contenu du RESUME_RECHERCHE.md :**
- 📋 Tableau récapitulatif de tous les articles scrapés
- 📖 Résumés courts (300 caractères) de chaque article
- 📊 Statistiques globales (total sections, liens, images, moyennes)
- 🔗 Liens vers chaque dossier d'article

### Exemple 2 : Mode interactif

```bash
cargo run

# > Choix : 2
# > Mot-clé : BMW
# > Nombre : 10  (ou Entrée pour défaut 5)
```

### Exemple 3 : Binaire compilé (plus rapide)

```bash
cargo build --release
./target/release/wikipedia_scraper -k "Rust" -n 3
```

## 📄 Structure des fichiers générés

### Par recherche (dossier parent)

```
resultats/Avion_20240116_143025/
├── RESUME_RECHERCHE.md          # ← Nouveau ! Résumé global
├── Avion.md                       # Article 1
├── Avion_de_ligne.md              # Article 2
└── Boeing_747.md                  # Article 3
```

### Par article (sous-dossier)

```
Avion/
├── data.json          # Toutes les données structurées en JSON
├── article.md         # Article formaté en Markdown
├── resume.txt         # Titre, URL et résumé
├── sections.txt       # Liste des sections (une par ligne)
├── liens.txt          # URLs des liens internes (une par ligne)
└── images.txt         # URLs des images (une par ligne)
```

## 🔧 Résolution des problèmes

### Erreur "Accès refusé" (Windows)

**Solution** : Déplacez le projet hors de OneDrive
```powershell
# Vers:
C:\Users\Admin\Documents\RUST\Scrappeur_wikipedia
```

### Erreur de compilation

```bash
cargo clean
cargo build
```

### Aucun résultat trouvé

- Vérifiez l'orthographe du mot-clé
- Essayez avec un mot-clé plus général
- Vérifiez votre connexion Internet

## 🎓 Cas d'usage

- 📚 Recherche documentaire automatisée
- 🔬 Collecte de données pour projets académiques
- 🤖 Construction de datasets pour ML/AI
- 📊 Analyse de contenu Wikipedia en masse

## 📝 Respect de Wikipedia

Ce scrappeur :
- ✅ Ajoute des pauses entre les requêtes (1 seconde)
- ✅ Utilise un User-Agent approprié
- ✅ Utilise l'API OpenSearch officielle
- ✅ Ne surcharge pas les serveurs

**Utilisez-le de manière responsable et éthique !**

## 🤝 Contribution

Projet ESGI - BAC +4 RUST - 2024

## 📞 Contact

Projet académique ESGI

---

**Note technique** : Ce projet implémente manuellement les requêtes HTTP tout en utilisant rustls uniquement pour la couche TLS/HTTPS (nécessaire car Wikipedia force HTTPS). Cela respecte l'esprit de la contrainte "pas de bibliothèque de haut niveau pour les requêtes réseau".
