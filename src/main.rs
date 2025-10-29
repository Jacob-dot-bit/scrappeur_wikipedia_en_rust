use clap::Parser;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use rustls::pki_types::ServerName;
use sanitize_filename::sanitize;

#[derive(Debug, Serialize, Deserialize)]
struct WikipediaPage {
    url: String,
    title: String,
    summary: String,
    sections: Vec<String>,
    links: Vec<String>,
    images: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(name = "Wikipedia Scraper")]
#[command(about = "Scrape des pages Wikipedia en français", long_about = None)]
struct Args {
    /// Fichier contenant la liste des URLs Wikipedia (une par ligne)
    #[arg(short, long)]
    fichier: Option<String>,

    /// URLs Wikipedia séparées par des virgules
    #[arg(short, long)]
    urls: Option<String>,

    /// Mot-clé à rechercher sur Wikipedia
    #[arg(short = 'k', long)]
    mot_cle: Option<String>,

    /// Nombre maximum de résultats à scraper (pour recherche par mot-clé)
    #[arg(short = 'n', long, default_value = "5")]
    nombre: usize,

    /// Dossier de sortie pour les résultats
    #[arg(short, long, default_value = "resultats")]
    output: String,
}

/// Fonction principale
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Récupérer la liste des URLs
    let urls = if let Some(mot_cle) = args.mot_cle.clone() {
        // Recherche par mot-clé
        println!("\n🔍 Recherche Wikipedia pour: \"{}\"", mot_cle);
        let resultats = rechercher_wikipedia(&mot_cle, args.nombre)?;
        
        if resultats.is_empty() {
            eprintln!("Aucun résultat trouvé pour \"{}\"", mot_cle);
            return Ok(());
        }
        
        println!("\n✓ {} résultat(s) trouvé(s):\n", resultats.len());
        for (i, url) in resultats.iter().enumerate() {
            println!("  {}. {}", i + 1, url);
        }
        println!();
        
        resultats
    } else if let Some(fichier) = args.fichier {
        // Lecture des URLs depuis un fichier
        let contenu = fs::read_to_string(fichier)?;
        let urls: Vec<String> = contenu.lines().map(|line| line.to_string()).collect();
        println!("\n📂 Chargement de {} URL(s) depuis le fichier", urls.len());
        urls
    } else if let Some(urls_str) = args.urls {
        // URLs fournies en ligne de commande
        urls_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Mode interactif
        get_urls_interactif(args.nombre)?
    };

    if urls.is_empty() {
        eprintln!("Erreur: Aucune URL fournie");
        return Ok(());
    }

    // Créer le dossier de sortie principal
    fs::create_dir_all(&args.output)?;

    // Créer un dossier spécifique pour cette recherche
    let search_folder = if let Some(mot_cle) = &args.mot_cle {
        // Recherche par mot-clé : créer un dossier avec le mot-clé et timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let folder_name = format!("{}_{}", sanitize(mot_cle), timestamp);
        format!("{}/{}", args.output, folder_name)
    } else if urls.len() > 1 {
        // Plusieurs URLs : créer un dossier avec timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("{}/batch_{}", args.output, timestamp)
    } else {
        // Une seule URL : pas de dossier parent supplémentaire
        args.output.clone()
    };

    fs::create_dir_all(&search_folder)?;

    println!("\n=== Scraping de {} page(s) ===\n", urls.len());
    println!("📁 Dossier de recherche : {}\n", search_folder);

    // Scraper chaque URL
    let mut scraped_articles = Vec::new();
    
    for (index, url) in urls.iter().enumerate() {
        println!("[{}/{}] Scraping de: {}", index + 1, urls.len(), url);

        match scrape_wikipedia(url) {
            Ok(page_data) => {
                // Créer un dossier pour cette page dans le dossier de recherche
                let page_folder = format!(
                    "{}/{}",
                    search_folder,
                    sanitize(&page_data.title)
                );
                fs::create_dir_all(&page_folder)?;

                // Sauvegarder les données
                save_page_data(&page_data, &page_folder)?;

                println!("  ✓ Titre: {}", page_data.title);
                println!("  ✓ Sections: {}", page_data.sections.len());
                println!("  ✓ Liens: {}", page_data.links.len());
                println!("  ✓ Images: {}", page_data.images.len());
                println!("  ✓ Sauvegardé dans: {}\n", page_folder);
                
                // Ajouter à la liste pour le résumé global
                scraped_articles.push(page_data);
            }
            Err(e) => {
                eprintln!("  ✗ Erreur: {}\n", e);
            }
        }

        // Pause entre les requêtes pour être respectueux
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Générer un fichier récapitulatif de la recherche
    if scraped_articles.len() > 1 {
        generate_search_summary(&scraped_articles, &search_folder, args.mot_cle.as_deref())?;
    }

    println!("=== Scraping terminé ===");
    println!("📂 Résultats disponibles dans: {}", search_folder);
    println!("📊 {} article(s) traité(s) avec succès", scraped_articles.len());

    Ok(())
}

/// Fonction pour rechercher des articles sur Wikipedia par mot-clé
fn rechercher_wikipedia(mot_cle: &str, max_resultats: usize) -> Result<Vec<String>, Box<dyn Error>> {
    // Encoder la requête pour la page Special:Recherche (espaces -> +, autres -> %XX)
    let mot_cle_encode = url_encode_query(mot_cle);

    // Construire le chemin exact demandé : /w/index.php?fulltext=1&search=...
    let search_page_path = format!("/w/index.php?fulltext=1&search={}", mot_cle_encode);

    println!("  Recherche via la page Special:Recherche...");
    match https_get("fr.wikipedia.org", &search_page_path) {
        Ok(html_content) => {
            let urls = extract_urls_from_search_html(&html_content, max_resultats);
            if urls.is_empty() {
                println!("  Aucun résultat trouvé sur la page de recherche pour '{}'.", mot_cle);
            }
            Ok(urls)
        }
        Err(e) => {
            println!("  ✗ Impossible de récupérer la page de recherche: {}", e);
            Ok(Vec::new())
        }
    }
}

fn extract_urls_from_search_html(html: &str, max_results: usize) -> Vec<String> {
    // Parse la page Special:Recherche et récupère les liens d'articles
    let mut urls = Vec::new();
    let document = Html::parse_document(html);

    // Sélecteurs courants pour les résultats de recherche
    let selector = Selector::parse(".mw-search-results .mw-search-result-heading a, .mw-search-result-heading a, a[href^='/wiki/']").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with("/wiki/") && !href.contains(":") && !href.contains('#') {
                let full = format!("https://fr.wikipedia.org{}", href);
                if !urls.contains(&full) {
                    urls.push(full);
                    if urls.len() >= max_results {
                        break;
                    }
                }
            }
        }
    }

    urls
}

// (OpenSearch parsing and legacy url_encode removed — we now use Special:Recherche and url_encode_query)

/// Fonction pour scraper une page Wikipedia
fn scrape_wikipedia(url: &str) -> Result<WikipediaPage, Box<dyn Error>> {
    let url_parts = parse_url(url)?;
    let host = &url_parts.0;
    let path = &url_parts.1;

    let html_content = http_get(host, path)?;
    let document = Html::parse_document(&html_content);

    // Extraire le titre
    let title_selector = Selector::parse("h1#firstHeading, h1.firstHeading").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Sans titre".to_string());

    // Extraire le résumé complet (tous les paragraphes du lead avant le premier h2)
    let summary = extract_summary_from_html(&html_content);

    // Extraire les sections
    let mut sections: Vec<String> = Vec::new();
    let section_selector1 = Selector::parse(".mw-headline").unwrap();
    for element in document.select(&section_selector1) {
        let section_text = element.text().collect::<String>().trim().to_string();
        if !section_text.is_empty() && section_text.len() > 1 {
            sections.push(section_text);
        }
    }

    // Extraire les liens internes
    let link_selector = Selector::parse("div#mw-content-text a[href^='/wiki/'], a[href^='/wiki/']").unwrap();
    let links: Vec<String> = document
        .select(&link_selector)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.contains(":") && !href.contains("#"))
        .take(50)
        .map(|href| format!("https://fr.wikipedia.org{}", href))
        .collect();

    // Extraire les images (filtrer les icônes)
    let image_selector = Selector::parse("img[src]").unwrap();
    let images: Vec<String> = document
        .select(&image_selector)
        .filter_map(|el| {
            let src = el.value().attr("src")?;
            let width = el.value().attr("width");
            let height = el.value().attr("height");
            
            if let (Some(w), Some(h)) = (width, height) {
                if let (Ok(w_num), Ok(h_num)) = (w.parse::<u32>(), h.parse::<u32>()) {
                    if w_num < 100 || h_num < 100 {
                        return None;
                    }
                }
            }
            
            if !(src.starts_with("//") || src.starts_with("http")) {
                return None;
            }
            
            if !(src.contains(".jpg") || src.contains(".jpeg") || 
                 src.contains(".png") || src.contains(".svg") || src.contains(".gif")) {
                return None;
            }
            
            if src.contains("/static/images/") || src.contains("/icons/") ||
               src.contains("Icon_") || src.contains("icon") || src.contains("logo") ||
               src.contains("20px-") || src.contains("15px-") {
                return None;
            }
            
            let img_url = if src.starts_with("//") {
                format!("https:{}", src)
            } else {
                src.to_string()
            };
            
            if img_url.contains("upload.wikimedia.org") {
                Some(img_url)
            } else {
                None
            }
        })
        .take(20)
        .collect();

    Ok(WikipediaPage {
        url: url.to_string(),
        title,
        summary,
        sections,
        links,
        images,
    })
}

fn extract_summary_from_html(html: &str) -> String {
    // Parser le document entier puis extraire l'HTML interne de div.mw-parser-output
    let doc = Html::parse_document(html);
    let container_selector = Selector::parse("div.mw-parser-output").unwrap();

    if let Some(container) = doc.select(&container_selector).next() {
        let inner = container.inner_html();
        let upto = if let Some(pos) = inner.find("<h2") {
            &inner[..pos]
        } else {
            &inner
        };

        let snippet = Html::parse_fragment(upto);
        let p_selector = Selector::parse("p").unwrap();
        let mut parts: Vec<String> = Vec::new();
        for p in snippet.select(&p_selector) {
            let text = p.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                parts.push(text);
            }
        }

        return parts.join("\n\n");
    }

    String::new()
}

fn http_get(host: &str, path: &str) -> Result<String, Box<dyn Error>> {
    if path.contains("wikipedia.org") || host.contains("wikipedia") {
        https_get(host, path)
    } else {
        https_get(host, path)
    }
}

fn https_get(host: &str, path: &str) -> Result<String, Box<dyn Error>> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = ServerName::try_from(host)?;
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name.to_owned())?;

    let addr = format!("{}:443", host);
    let mut sock = TcpStream::connect(&addr)
        .map_err(|e| format!("Connexion impossible à {}: {}", host, e))?;

    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\r\n\
         Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
         Accept-Language: fr,fr-FR;q=0.8,en-US;q=0.5,en;q=0.3\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );

    while conn.is_handshaking() {
        conn.complete_io(&mut sock)?;
    }

    conn.writer().write_all(request.as_bytes())?;
    conn.complete_io(&mut sock)?;

    let mut response = Vec::new();
    loop {
        let mut buf = vec![0u8; 8192];
        match conn.reader().read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                conn.complete_io(&mut sock)?;
            }
            Err(e) => return Err(e.into()),
        }
        
        if let Err(e) = conn.complete_io(&mut sock) {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                break;
            }
        }
    }
    
    let response_str = String::from_utf8_lossy(&response).to_string();

    let status_line = response_str.lines().next().unwrap_or("");
    
    if status_line.contains("301") || status_line.contains("302") {
        if let Some(location) = extract_header(&response_str, "Location") {
            if let Ok((new_host, new_path)) = parse_url(&location) {
                return https_get(&new_host, &new_path);
            }
        }
    }

    if !status_line.contains("200") {
        return Err(format!("Erreur HTTP: {}", status_line).into());
    }

    if let Some(body_start) = response_str.find("\r\n\r\n") {
        Ok(response_str[body_start + 4..].to_string())
    } else if let Some(body_start) = response_str.find("\n\n") {
        Ok(response_str[body_start + 2..].to_string())
    } else {
        Err("Impossible de séparer headers et body".into())
    }
}

fn extract_header(response: &str, header_name: &str) -> Option<String> {
    let header_prefix = format!("{}: ", header_name);
    
    for line in response.lines() {
        if line.starts_with(&header_prefix) || line.to_lowercase().starts_with(&header_prefix.to_lowercase()) {
            return Some(line[header_prefix.len()..].trim().to_string());
        }
    }
    
    None
}

fn parse_url(url: &str) -> Result<(String, String), Box<dyn Error>> {
    let url = url.trim();

    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    if let Some(pos) = url.find('/') {
        let host = url[..pos].to_string();
        let path = url[pos..].to_string();
        Ok((host, path))
    } else {
        Ok((url.to_string(), "/".to_string()))
    }
}

/// Fonction pour sauvegarder les données d'une page
fn save_page_data(page: &WikipediaPage, folder: &str) -> Result<(), Box<dyn Error>> {
    let json_path = format!("{}/data.json", folder);
    let json = serde_json::to_string_pretty(page)?;
    fs::write(&json_path, json)?;

    let markdown_path = format!("{}/article.md", folder);
    let markdown_content = generate_markdown(page);
    fs::write(&markdown_path, markdown_content)?;

    let summary_path = format!("{}/resume.txt", folder);
    let summary_content = format!(
        "Titre: {}\n\nURL: {}\n\nRésumé:\n{}\n",
        page.title, page.url, page.summary
    );
    fs::write(&summary_path, summary_content)?;

    let sections_path = format!("{}/sections.txt", folder);
    let sections_content = page.sections.join("\n");
    fs::write(&sections_path, sections_content)?;

    let links_path = format!("{}/liens.txt", folder);
    let links_content = page.links.join("\n");
    fs::write(&links_path, links_content)?;

    let images_path = format!("{}/images.txt", folder);
    let images_content = page.images.join("\n");
    fs::write(&images_path, images_content)?;

    Ok(())
}

fn generate_markdown(page: &WikipediaPage) -> String {
    let mut markdown = String::new();
    
    markdown.push_str(&format!("# {}\n\n", page.title));
    markdown.push_str(&format!("**Source:** [Wikipedia]({})  \n", page.url));
    markdown.push_str(&format!("**Date:** {}  \n\n", 
        chrono::Local::now().format("%d/%m/%Y à %H:%M:%S")));
    
    markdown.push_str("## Résumé\n\n");
    if !page.summary.is_empty() {
        markdown.push_str(&page.summary);
        markdown.push_str("\n\n");
    } else {
        markdown.push_str("*Résumé non disponible*\n\n");
    }
    
    if !page.sections.is_empty() {
        markdown.push_str("## Sections\n\n");
        for section in &page.sections {
            markdown.push_str(&format!("- {}\n", section));
        }
        markdown.push_str("\n");
    }
    
    markdown
}

/// Fonction pour le mode interactif (saisie des URLs par l'utilisateur)
fn get_urls_interactif(default_nombre: usize) -> Result<Vec<String>, Box<dyn Error>> {
    println!("\n=== Scraper Wikipedia (Mode interactif) ===\n");
    println!("Choisissez une option :");
    println!("1. Entrer des URLs directement");
    println!("2. Rechercher par mot-clé");
    
    print!("\nVotre choix (1-2) : ");
    io::stdout().flush()?;
    
    let mut choix = String::new();
    io::stdin().read_line(&mut choix)?;
    
    match choix.trim() {
        "1" => {
            println!("\nEntrez les URLs Wikipedia (une par ligne)");
            println!("Appuyez sur Ctrl+D (Linux/Mac) ou Ctrl+Z puis Entrée (Windows) pour terminer\n");

            let mut urls = Vec::new();
            
            loop {
                let mut url = String::new();
                match io::stdin().read_line(&mut url) {
                    Ok(0) => break, // EOF (Ctrl+D ou Ctrl+Z)
                    Ok(_) => {
                        let url = url.trim();
                        if !url.is_empty() {
                            urls.push(url.to_string());
                            println!("  [{}] Ajouté: {}", urls.len(), url);
                        }
                    }
                    Err(_) => break,
                }
            }
            
            Ok(urls)
        }
        "2" => {
            print!("Entrez le mot-clé à rechercher : ");
            io::stdout().flush()?;
            
            let mut mot_cle = String::new();
            io::stdin().read_line(&mut mot_cle)?;
            let mot_cle = mot_cle.trim();
            
            print!("Nombre de résultats à scraper (défaut: {}, max 20) : ", default_nombre);
            io::stdout().flush()?;
            
            let mut nombre_str = String::new();
            io::stdin().read_line(&mut nombre_str)?;
            
            let nombre = if nombre_str.trim().is_empty() {
                default_nombre
            } else {
                nombre_str.trim().parse::<usize>().unwrap_or(default_nombre).min(20)
            };
            
            println!("\n🔍 Recherche en cours de \"{}\" ({} résultats)...\n", mot_cle, nombre);
            rechercher_wikipedia(mot_cle, nombre)
        }
        _ => {
            println!("Choix invalide");
            Ok(Vec::new())
        }
    }
}

/// Fonction pour générer un résumé de la recherche
fn generate_search_summary(
    articles: &[WikipediaPage], 
    folder: &str, 
    search_term: Option<&str>
) -> Result<(), Box<dyn Error>> {
    let summary_path = format!("{}/RESUME_RECHERCHE.md", folder);
    let mut summary = String::new();
    
    // En-tête
    if let Some(term) = search_term {
        summary.push_str(&format!("# 🔍 Résumé de recherche : \"{}\"\n\n", term));
    } else {
        summary.push_str("# 📚 Résumé de scraping\n\n");
    }
    
    summary.push_str(&format!("**Date** : {}\n\n", 
        chrono::Local::now().format("%d/%m/%Y à %H:%M:%S")));
    summary.push_str(&format!("**Nombre d'articles** : {}\n\n", articles.len()));
    
    summary.push_str("---\n\n");
    
    // Table des matières
    summary.push_str("## 📋 Articles scrapés\n\n");
    summary.push_str("| # | Article | Sections | Liens | Images | Dossier |\n");
    summary.push_str("|---|---------|----------|-------|--------|----------|\n");
    
    for (i, article) in articles.iter().enumerate() {
        let folder_name = sanitize(&article.title);
        summary.push_str(&format!(
            "| {} | [{}]({}) | {} | {} | {} | [📁](./{}) |\n",
            i + 1,
            article.title,
            article.url,
            article.sections.len(),
            article.links.len(),
            article.images.len(),
            folder_name
        ));
    }
    
    summary.push_str("\n---\n\n");
    
    // Résumés courts de chaque article
    summary.push_str("## 📖 Résumés des articles\n\n");
    
    for (i, article) in articles.iter().enumerate() {
        summary.push_str(&format!("### {}. {}\n\n", i + 1, article.title));
        summary.push_str(&format!("**URL** : [{}]({})\n\n", article.title, article.url));
        
        if !article.summary.is_empty() {
            // Prendre les 300 premiers caractères du résumé
            let short_summary = if article.summary.len() > 300 {
                format!("{}...", &article.summary[..297])
            } else {
                article.summary.clone()
            };
            summary.push_str(&format!("{}\n\n", short_summary));
            summary.push_str(&format!("> 📄 [Lire l'article complet](./{}/article.md)\n\n", sanitize(&article.title)));
        } else {
            summary.push_str("*Résumé non disponible*\n\n");
            summary.push_str(&format!("> 📄 [Consulter les données](./{}/)\n\n", sanitize(&article.title)));
        }
    
        // Sections principales
        if !article.sections.is_empty() {
            summary.push_str("**Sections principales** : ");
            let sections_preview: Vec<String> = article.sections.iter().take(5).cloned().collect();
            summary.push_str(&sections_preview.join(", "));
            if article.sections.len() > 5 {
                summary.push_str(&format!(" (et {} autres...)", article.sections.len() - 5));
            }
            summary.push_str("\n\n");
        }
        
        summary.push_str("---\n\n");
    }
    
    // Statistiques globales
    summary.push_str("## 📊 Statistiques globales\n\n");
    summary.push_str("```\n");
    summary.push_str(&format!("Total articles       : {}\n", articles.len()));
    summary.push_str(&format!("Total sections       : {}\n", articles.iter().map(|a| a.sections.len()).sum::<usize>()));
    summary.push_str(&format!("Total liens          : {}\n", articles.iter().map(|a| a.links.len()).sum::<usize>()));
    summary.push_str(&format!("Total images         : {}\n", articles.iter().map(|a| a.images.len()).sum::<usize>()));
    
    let avg_sections = articles.iter().map(|a| a.sections.len()).sum::<usize>() as f64 / articles.len() as f64;
    summary.push_str(&format!("Moyenne sections     : {:.1}\n", avg_sections));
    
    let total_chars: usize = articles.iter().map(|a| a.summary.len()).sum();
    summary.push_str(&format!("Total caractères     : {}\n", total_chars));
    summary.push_str("```\n\n");
    
    // Footer
    summary.push_str("---\n\n");
    summary.push_str("*Résumé généré automatiquement par le Scrappeur Wikipedia en Rust*\n");
    summary.push_str("*ESGI - BAC +4 RUST*\n");
    
    fs::write(&summary_path, summary)?;
    println!("\n📄 Résumé de recherche généré : {}", summary_path);
    
    Ok(())
}

/// Encodage pour les paramètres de requête (search=)
fn url_encode_query(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c.to_string(),
            ' ' => "+".to_string(),
            '-' | '_' | '.' | '~' => c.to_string(),
            _ => {
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                encoded.bytes().map(|b| format!("%{:02X}", b)).collect::<String>()
            }
        })
        .collect()
}